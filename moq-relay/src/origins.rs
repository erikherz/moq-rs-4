use std::{
	collections::{HashMap, VecDeque},
	sync::{Arc, Mutex},
};

use moq_transfork::prelude::*;

#[derive(Default)]
struct RouterState {
	lookup: HashMap<String, VecDeque<BroadcastConsumer>>,
}

#[derive(Clone, Default)]
pub struct Origins {
	state: Arc<Mutex<RouterState>>,
}

impl Origins {
	pub fn new() -> Self {
		let state = Arc::new(Mutex::new(RouterState::default()));
		Origins { state }
	}

	pub fn announce(&self, broadcast: BroadcastConsumer) -> RouterAnnounce {
		let mut state = self.state.lock().unwrap();
		let broadcasts = state.lookup.entry(broadcast.name.clone()).or_default();

		// push_front so newest announce wins.
		broadcasts.push_front(broadcast.clone());
		RouterAnnounce::new(self.clone(), broadcast)
	}

	fn unannounce(&self, broadcast: &BroadcastConsumer) {
		let mut state = self.state.lock().unwrap();
		let _broadcasts = state.lookup.get_mut(&broadcast.name).expect("missing entry");

		// TODO? broadcasts.retain(|b| b != broadcast);
		unimplemented!("unannounce");
	}

	pub async fn serve(&self, writer: &mut RouterProducer<Broadcast>) {
		while let Some(request) = writer.requested().await {
			let state = self.state.lock().unwrap();
			match state.lookup.get(&request.info.name).and_then(|bs| bs.front()) {
				Some(broadcast) => request.serve(broadcast.clone()),
				None => request.close(moq_transfork::Error::NotFound),
			}
		}
	}
}

pub struct RouterAnnounce {
	router: Origins,
	broadcast: BroadcastConsumer,
}

impl RouterAnnounce {
	pub fn new(router: Origins, broadcast: BroadcastConsumer) -> Self {
		Self { router, broadcast }
	}
}

impl Drop for RouterAnnounce {
	fn drop(&mut self) {
		self.router.unannounce(&self.broadcast);
	}
}