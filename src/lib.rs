#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod broadcast;
pub mod broadcast_activity_view_controller;
pub mod broadcast_configuration;
pub mod broadcast_controller;
pub mod broadcast_extension;
pub mod broadcast_handler;
pub mod broadcast_sample_handler;
pub mod error;
mod ffi;
pub mod preview_view;
mod private;
pub mod recorder;
pub mod sample_buffer_delegate;
pub mod screen_recorder;
pub mod system_broadcast_picker_view;

pub use broadcast_activity_view_controller::{
    BroadcastActivityControllerHandle, BroadcastActivityViewController,
};
pub use broadcast_configuration::BroadcastConfiguration;
pub use broadcast_controller::{
    BroadcastController, BroadcastControllerEvent, BroadcastControllerObserver,
};
pub use broadcast_extension::{
    BroadcastExtensionContext, BroadcastingApplicationInfo,
    RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};
pub use broadcast_handler::BroadcastHandler;
pub use broadcast_sample_handler::BroadcastSampleHandler;
pub use error::{
    RecordingErrorCode, ReplayKitError, ReplayKitFrameworkError, RP_RECORDING_ERROR_DOMAIN,
    SC_STREAM_ERROR_DOMAIN,
};
pub use preview_view::{PreviewEvent, PreviewViewController, PreviewViewControllerObserver};
pub use sample_buffer_delegate::{
    CaptureEvent, CaptureSample, SampleBufferCaptureSession, SampleBufferDelegate, SampleBufferType,
};
pub use screen_recorder::{
    CameraPosition, CameraPreviewView, DetailedRecordingEvent, DetailedRecordingObserver,
    RecordingEvent, RecordingObserver, ScreenRecorder, ScreenRecorderState,
};
pub use system_broadcast_picker_view::SystemBroadcastPickerView;

/// Common imports.
pub mod prelude {
    pub use crate::broadcast_activity_view_controller::{
        BroadcastActivityControllerHandle, BroadcastActivityViewController,
    };
    pub use crate::broadcast_configuration::BroadcastConfiguration;
    pub use crate::broadcast_controller::{
        BroadcastController, BroadcastControllerEvent, BroadcastControllerObserver,
    };
    pub use crate::broadcast_extension::{
        BroadcastExtensionContext, BroadcastingApplicationInfo,
        RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
    };
    pub use crate::broadcast_handler::BroadcastHandler;
    pub use crate::broadcast_sample_handler::BroadcastSampleHandler;
    pub use crate::error::{
        RecordingErrorCode, ReplayKitError, ReplayKitFrameworkError, RP_RECORDING_ERROR_DOMAIN,
        SC_STREAM_ERROR_DOMAIN,
    };
    pub use crate::preview_view::{
        PreviewEvent, PreviewViewController, PreviewViewControllerObserver,
    };
    pub use crate::sample_buffer_delegate::{
        CaptureEvent, CaptureSample, SampleBufferCaptureSession, SampleBufferDelegate,
        SampleBufferType,
    };
    pub use crate::screen_recorder::{
        CameraPosition, CameraPreviewView, DetailedRecordingEvent, DetailedRecordingObserver,
        RecordingEvent, RecordingObserver, ScreenRecorder, ScreenRecorderState,
    };
    pub use crate::system_broadcast_picker_view::SystemBroadcastPickerView;
}
