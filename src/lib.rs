#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod broadcast;
pub mod error;
mod ffi;
mod private;
pub mod recorder;

pub use broadcast::{BroadcastActivityControllerHandle, BroadcastController};
pub use error::ReplayKitError;
pub use recorder::{RecordingObserver, ScreenRecorder};

/// Common imports.
pub mod prelude {
    pub use crate::broadcast::{BroadcastActivityControllerHandle, BroadcastController};
    pub use crate::error::ReplayKitError;
    pub use crate::recorder::{RecordingObserver, ScreenRecorder};
}
