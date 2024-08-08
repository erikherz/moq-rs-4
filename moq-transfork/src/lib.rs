//! An implementation of the MoQ Transport protocol.
//!
//! MoQ Transport is a pub/sub protocol over QUIC.
//! While originally designed for live media, MoQ Transport is generic and can be used for other live applications.
//! The specification is a work in progress and will change.
//! See the [specification](https://datatracker.ietf.org/doc/draft-ietf-moq-transport/) and [github](https://github.com/moq-wg/moq-transport) for any updates.
pub mod coding;
pub mod error;
pub mod message;
pub mod model;
pub mod prelude;
pub mod session;
pub mod setup;
pub(crate) mod util;

pub use error::*;
pub use model::*;
pub use session::*;
pub use setup::Role;