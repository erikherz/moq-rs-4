//! A broadcast is a collection of tracks, split into two handles: [Writer] and [Reader].
//!
//! The [Writer] can create tracks, either manually or on request.
//! It receives all requests by a [Reader] for a tracks that don't exist.
//! The simplest implementation is to close every unknown track with [ServeError::NotFound].
//!
//! A [Reader] can request tracks by name.
//! If the track already exists, it will be returned.
//! If the track doesn't exist, it will be sent to [Unknown] to be handled.
//! A [Reader] can be cloned to create multiple subscriptions.
//!
//! The broadcast is automatically closed with [ServeError::Done] when [Writer] is dropped, or all [Reader]s are dropped.
use std::{collections::HashMap, fmt, ops, sync::Arc};

use super::{Track, TrackBuilder, TrackReader, TrackWriter, UnknownReader};
use crate::{util::State, Closed};

/// Static information about a broadcast.
#[derive(Clone)]
pub struct Broadcast {
	pub name: String,
}

impl Broadcast {
	pub fn new(name: &str) -> Self {
		Self { name: name.to_string() }
	}

	pub fn produce(self) -> (BroadcastWriter, BroadcastReader) {
		let info = Arc::new(self);
		let state = State::default();

		let writer = BroadcastWriter::new(state.split(), info.clone());
		let reader = BroadcastReader::new(state, info);

		(writer, reader)
	}
}

pub struct BroadcastState {
	tracks: HashMap<String, TrackReader>,
	unknown: Option<UnknownReader>,
	closed: Result<(), Closed>,
}

impl Default for BroadcastState {
	fn default() -> Self {
		Self {
			tracks: HashMap::new(),
			unknown: None,
			closed: Ok(()),
		}
	}
}

/// Publish new tracks for a broadcast by name.
pub struct BroadcastWriter {
	state: State<BroadcastState>,
	pub info: Arc<Broadcast>,
}

impl BroadcastWriter {
	fn new(state: State<BroadcastState>, info: Arc<Broadcast>) -> Self {
		Self { state, info }
	}

	pub fn create(&mut self, name: &str, priority: u64) -> BroadcastTrackBuilder {
		BroadcastTrackBuilder::new(self, name, priority)
	}

	/// Optionally route unknown tracks to the provided [UnknownReader].
	pub fn unknown(&mut self, reader: UnknownReader) -> Result<(), Closed> {
		self.state.lock_mut().ok_or(Closed::Cancel)?.unknown = Some(reader);
		Ok(())
	}

	/// Insert a track into the broadcast.
	pub fn insert(&mut self, track: Track) -> Result<TrackWriter, Closed> {
		let (writer, reader) = track.produce();

		// NOTE: We overwrite the track if it already exists.
		self.state
			.lock_mut()
			.ok_or(Closed::Cancel)?
			.tracks
			.insert(reader.name.clone(), reader);

		Ok(writer)
	}

	pub fn remove(&mut self, track: &str) -> Option<TrackReader> {
		self.state.lock_mut()?.tracks.remove(track)
	}

	pub fn close(&mut self, code: u32) -> Result<(), Closed> {
		let state = self.state.lock();
		state.closed.clone()?;
		state.into_mut().ok_or(Closed::Cancel)?.closed = Err(Closed::App(code));

		Ok(())
	}

	pub async fn closed(&self) -> Result<(), Closed> {
		loop {
			{
				let state = self.state.lock();
				state.closed.clone()?;

				match state.modified() {
					Some(notify) => notify,
					None => return Ok(()),
				}
			}
			.await
		}
	}
}

impl ops::Deref for BroadcastWriter {
	type Target = Broadcast;

	fn deref(&self) -> &Self::Target {
		&self.info
	}
}

pub struct BroadcastTrackBuilder<'a> {
	broadcast: &'a mut BroadcastWriter,
	track: TrackBuilder,
}

impl<'a> BroadcastTrackBuilder<'a> {
	fn new(broadcast: &'a mut BroadcastWriter, name: &str, priority: u64) -> Self {
		Self {
			track: Track::new(&broadcast.name, name, priority),
			broadcast,
		}
	}

	pub fn build(self) -> Result<TrackWriter, Closed> {
		self.broadcast.insert(self.track.build())
	}
}

impl<'a> ops::Deref for BroadcastTrackBuilder<'a> {
	type Target = TrackBuilder;

	fn deref(&self) -> &TrackBuilder {
		&self.track
	}
}

impl<'a> ops::DerefMut for BroadcastTrackBuilder<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.track
	}
}

/// Subscribe to a broadcast by requesting tracks.
///
/// This can be cloned to create handles.
#[derive(Clone)]
pub struct BroadcastReader {
	state: State<BroadcastState>,
	pub info: Arc<Broadcast>,
}

impl BroadcastReader {
	fn new(state: State<BroadcastState>, info: Arc<Broadcast>) -> Self {
		Self { state, info }
	}

	/// Get a track from the broadcast by name.
	pub async fn subscribe(&mut self, track: Track) -> Result<TrackReader, Closed> {
		let unknown = {
			let state = self.state.lock();
			if let Some(track) = state.tracks.get(&track.name).cloned() {
				return Ok(track);
			}

			state.unknown.clone().ok_or(Closed::UnknownTrack)?
		};

		// TODO cache to deduplicate?
		unknown.subscribe(track).await
	}

	pub async fn closed(&self) -> Result<(), Closed> {
		loop {
			{
				let state = self.state.lock();
				state.closed.clone()?;

				match state.modified() {
					Some(notify) => notify,
					None => return Ok(()),
				}
			}
			.await
		}
	}
}

impl ops::Deref for BroadcastReader {
	type Target = Broadcast;

	fn deref(&self) -> &Self::Target {
		&self.info
	}
}
