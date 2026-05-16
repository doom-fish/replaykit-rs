#![allow(missing_docs, dead_code)]

pub mod broadcast_activity_view_controller;
pub mod broadcast_configuration;
pub mod broadcast_controller;
pub mod core;
pub mod preview_view;
pub mod sample_buffer_delegate;
pub mod screen_recorder;
pub mod system_broadcast_picker_view;

pub use broadcast_activity_view_controller::*;
pub use broadcast_configuration::*;
pub use broadcast_controller::*;
pub use core::*;
pub use preview_view::*;
pub use sample_buffer_delegate::*;
pub use screen_recorder::*;
pub use system_broadcast_picker_view::*;

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const TIMED_OUT: i32 = -2;
    pub const NOT_SUPPORTED: i32 = -3;
    pub const FRAMEWORK_ERROR: i32 = -4;
    pub const UNKNOWN: i32 = -99;
}
