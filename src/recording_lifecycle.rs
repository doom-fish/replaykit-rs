use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::{RecorderStateError, RecordingErrorCode, ReplayKitError};
use crate::ffi;
use crate::screen_recorder::ScreenRecorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordingPhase {
    Idle,
    Starting,
    Recording,
    Stopping,
    Stopped,
    Discarding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingOperation {
    Start,
    Stop,
    StopToOutput,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Starting { abandoned: bool },
    Recording,
    Stopping,
    Stopped,
    Discarding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Observation {
    is_recording: bool,
    abandoned_start_pending: bool,
}

impl State {
    const fn phase(self) -> RecordingPhase {
        match self {
            Self::Idle => RecordingPhase::Idle,
            Self::Starting { .. } => RecordingPhase::Starting,
            Self::Recording => RecordingPhase::Recording,
            Self::Stopping => RecordingPhase::Stopping,
            Self::Stopped => RecordingPhase::Stopped,
            Self::Discarding => RecordingPhase::Discarding,
        }
    }

    const fn resync(self, observation: Observation) -> Self {
        match self {
            Self::Starting { abandoned: true } if !observation.abandoned_start_pending => {
                if observation.is_recording {
                    Self::Recording
                } else {
                    Self::Idle
                }
            }
            Self::Idle | Self::Stopped if observation.is_recording => Self::Recording,
            Self::Recording if !observation.is_recording => Self::Stopped,
            other => other,
        }
    }

    const fn begin(self, operation: RecordingOperation) -> Result<Self, RecorderStateError> {
        match (self, operation) {
            (Self::Starting { .. } | Self::Stopping | Self::Discarding, _) => {
                Err(RecorderStateError::OperationInProgress)
            }
            (Self::Idle | Self::Stopped, RecordingOperation::Start) => {
                Ok(Self::Starting { abandoned: false })
            }
            (Self::Recording, RecordingOperation::Start | RecordingOperation::Discard) => {
                Err(RecorderStateError::RecordingInProgress)
            }
            (Self::Recording, RecordingOperation::Stop | RecordingOperation::StopToOutput) => {
                Ok(Self::Stopping)
            }
            (Self::Stopped, RecordingOperation::Discard) => Ok(Self::Discarding),
            (
                Self::Idle | Self::Stopped,
                RecordingOperation::Stop | RecordingOperation::StopToOutput,
            )
            | (Self::Idle, RecordingOperation::Discard) => Err(RecorderStateError::NoRecording),
        }
    }

    fn after(
        operation: RecordingOperation,
        outcome: Result<(), &ReplayKitError>,
        observation: Observation,
    ) -> Self {
        match (operation, outcome) {
            (RecordingOperation::Start, Ok(())) => Self::Recording,
            (RecordingOperation::Start, Err(ReplayKitError::TimedOut(_))) => {
                Self::Starting { abandoned: true }
            }
            (RecordingOperation::Stop, Ok(())) => Self::Stopped,
            (RecordingOperation::StopToOutput | RecordingOperation::Discard, Ok(()))
            | (RecordingOperation::Discard, Err(_)) => Self::Idle,
            (RecordingOperation::Stop | RecordingOperation::StopToOutput, Err(error))
                if stopped_without_a_recording(error) =>
            {
                Self::Idle
            }
            (RecordingOperation::Start, Err(_)) => {
                if observation.is_recording {
                    Self::Recording
                } else {
                    Self::Idle
                }
            }
            (RecordingOperation::Stop | RecordingOperation::StopToOutput, Err(_)) => {
                if observation.is_recording {
                    Self::Recording
                } else {
                    Self::Stopped
                }
            }
        }
    }
}

fn stopped_without_a_recording(error: &ReplayKitError) -> bool {
    matches!(
        error,
        ReplayKitError::Framework(error)
            if error.recording_code() == Some(RecordingErrorCode::AttemptToStopNonRecording)
    )
}

static STATE: Mutex<State> = Mutex::new(State::Idle);

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

fn observe(recorder: &ScreenRecorder) -> Observation {
    Observation {
        is_recording: recorder.is_recording(),
        abandoned_start_pending: unsafe { ffi::rk_screen_recorder_abandoned_start_pending() },
    }
}

pub fn begin(
    recorder: &ScreenRecorder,
    operation: RecordingOperation,
) -> Result<(), ReplayKitError> {
    let mut state = state();
    let current = state.resync(observe(recorder));
    let next = current.begin(operation);
    *state = next.unwrap_or(current);
    drop(state);
    next.map(|_| ()).map_err(ReplayKitError::InvalidState)
}

pub fn finish(operation: RecordingOperation, outcome: Result<(), &ReplayKitError>) {
    let observation = ScreenRecorder::shared().map_or(
        Observation {
            is_recording: false,
            abandoned_start_pending: false,
        },
        |recorder| observe(&recorder),
    );
    *state() = State::after(operation, outcome, observation);
}

#[cfg(all(test, feature = "async"))]
pub fn reset() {
    *state() = State::Idle;
}

impl ScreenRecorder {
    pub fn recording_phase(&self) -> RecordingPhase {
        let mut state = state();
        *state = state.resync(observe(self));
        state.phase()
    }
}

#[cfg(test)]
mod tests {
    use super::{Observation, RecordingOperation, RecordingPhase, State};
    use crate::error::{
        RecorderStateError, ReplayKitError, ReplayKitFrameworkError, RP_RECORDING_ERROR_DOMAIN,
    };

    const NOT_RECORDING: Observation = Observation {
        is_recording: false,
        abandoned_start_pending: false,
    };
    const RECORDING: Observation = Observation {
        is_recording: true,
        abandoned_start_pending: false,
    };
    const START_PENDING: Observation = Observation {
        is_recording: false,
        abandoned_start_pending: true,
    };

    fn framework_error(code: i64) -> ReplayKitError {
        ReplayKitError::Framework(ReplayKitFrameworkError {
            domain: RP_RECORDING_ERROR_DOMAIN.into(),
            code,
            localized_description: String::new(),
        })
    }

    fn timed_out() -> ReplayKitError {
        ReplayKitError::TimedOut(String::new())
    }

    #[test]
    fn a_recording_cycle_walks_every_phase() {
        let starting = State::Idle
            .resync(NOT_RECORDING)
            .begin(RecordingOperation::Start)
            .expect("start from idle");
        assert_eq!(starting.phase(), RecordingPhase::Starting);

        let recording = State::after(RecordingOperation::Start, Ok(()), RECORDING);
        assert_eq!(recording.phase(), RecordingPhase::Recording);

        let stopping = recording
            .resync(RECORDING)
            .begin(RecordingOperation::Stop)
            .expect("stop while recording");
        assert_eq!(stopping.phase(), RecordingPhase::Stopping);

        let stopped = State::after(RecordingOperation::Stop, Ok(()), NOT_RECORDING);
        assert_eq!(stopped.phase(), RecordingPhase::Stopped);

        let discarding = stopped
            .resync(NOT_RECORDING)
            .begin(RecordingOperation::Discard)
            .expect("discard after stop");
        assert_eq!(discarding.phase(), RecordingPhase::Discarding);

        let idle = State::after(RecordingOperation::Discard, Ok(()), NOT_RECORDING);
        assert_eq!(idle.phase(), RecordingPhase::Idle);
    }

    #[test]
    fn calls_that_are_invalid_in_the_current_phase_are_rejected() {
        let cases = [
            (State::Idle, NOT_RECORDING, RecordingOperation::Stop, RecorderStateError::NoRecording),
            (State::Idle, NOT_RECORDING, RecordingOperation::StopToOutput, RecorderStateError::NoRecording),
            (State::Idle, NOT_RECORDING, RecordingOperation::Discard, RecorderStateError::NoRecording),
            (State::Stopped, NOT_RECORDING, RecordingOperation::Stop, RecorderStateError::NoRecording),
            (State::Stopped, NOT_RECORDING, RecordingOperation::StopToOutput, RecorderStateError::NoRecording),
            (State::Recording, RECORDING, RecordingOperation::Start, RecorderStateError::RecordingInProgress),
            (State::Recording, RECORDING, RecordingOperation::Discard, RecorderStateError::RecordingInProgress),
            (State::Idle, RECORDING, RecordingOperation::Discard, RecorderStateError::RecordingInProgress),
            (State::Starting { abandoned: false }, NOT_RECORDING, RecordingOperation::Stop, RecorderStateError::OperationInProgress),
            (State::Starting { abandoned: false }, NOT_RECORDING, RecordingOperation::Discard, RecorderStateError::OperationInProgress),
            (State::Stopping, RECORDING, RecordingOperation::Stop, RecorderStateError::OperationInProgress),
            (State::Stopping, NOT_RECORDING, RecordingOperation::Discard, RecorderStateError::OperationInProgress),
            (State::Discarding, NOT_RECORDING, RecordingOperation::Discard, RecorderStateError::OperationInProgress),
            (State::Discarding, NOT_RECORDING, RecordingOperation::Start, RecorderStateError::OperationInProgress),
            (State::Starting { abandoned: true }, START_PENDING, RecordingOperation::Start, RecorderStateError::OperationInProgress),
            (State::Starting { abandoned: true }, START_PENDING, RecordingOperation::Stop, RecorderStateError::OperationInProgress),
        ];
        for (state, observation, operation, expected) in cases {
            assert_eq!(
                state.resync(observation).begin(operation),
                Err(expected),
                "{operation:?} in {state:?} with {observation:?}"
            );
        }
    }

    #[test]
    fn valid_calls_move_to_their_in_flight_phase() {
        let cases = [
            (State::Idle, NOT_RECORDING, RecordingOperation::Start, RecordingPhase::Starting),
            (State::Stopped, NOT_RECORDING, RecordingOperation::Start, RecordingPhase::Starting),
            (State::Recording, RECORDING, RecordingOperation::Stop, RecordingPhase::Stopping),
            (State::Recording, RECORDING, RecordingOperation::StopToOutput, RecordingPhase::Stopping),
            (State::Idle, RECORDING, RecordingOperation::Stop, RecordingPhase::Stopping),
            (State::Stopped, NOT_RECORDING, RecordingOperation::Discard, RecordingPhase::Discarding),
            (State::Recording, NOT_RECORDING, RecordingOperation::Discard, RecordingPhase::Discarding),
        ];
        for (state, observation, operation, expected) in cases {
            let next = state
                .resync(observation)
                .begin(operation)
                .unwrap_or_else(|error| panic!("{operation:?} in {state:?}: {error}"));
            assert_eq!(next.phase(), expected, "{operation:?} in {state:?}");
        }
    }

    #[test]
    fn resync_follows_recordings_started_or_ended_elsewhere() {
        assert_eq!(State::Idle.resync(RECORDING), State::Recording);
        assert_eq!(State::Stopped.resync(RECORDING), State::Recording);
        assert_eq!(State::Recording.resync(NOT_RECORDING), State::Stopped);
        assert_eq!(State::Idle.resync(NOT_RECORDING), State::Idle);
        assert_eq!(State::Stopping.resync(NOT_RECORDING), State::Stopping);
        assert_eq!(State::Discarding.resync(RECORDING), State::Discarding);
        assert_eq!(
            State::Starting { abandoned: false }.resync(NOT_RECORDING),
            State::Starting { abandoned: false }
        );
    }

    #[test]
    fn an_abandoned_start_blocks_calls_until_its_cleanup_finishes() {
        let abandoned = State::after(RecordingOperation::Start, Err(&timed_out()), NOT_RECORDING);
        assert_eq!(abandoned, State::Starting { abandoned: true });
        assert_eq!(abandoned.resync(START_PENDING), abandoned);
        assert_eq!(abandoned.resync(NOT_RECORDING), State::Idle);
        assert_eq!(abandoned.resync(RECORDING), State::Recording);
    }

    #[test]
    fn completions_settle_the_phase() {
        let not_recording = framework_error(-5829);
        let failed = framework_error(-5804);
        let timed_out = timed_out();
        let cases = [
            (RecordingOperation::Start, Err(&failed), NOT_RECORDING, State::Idle),
            (RecordingOperation::Start, Err(&failed), RECORDING, State::Recording),
            (RecordingOperation::Stop, Err(&not_recording), RECORDING, State::Idle),
            (RecordingOperation::Stop, Err(&timed_out), RECORDING, State::Recording),
            (RecordingOperation::Stop, Err(&timed_out), NOT_RECORDING, State::Stopped),
            (RecordingOperation::Stop, Err(&failed), NOT_RECORDING, State::Stopped),
            (RecordingOperation::StopToOutput, Ok(()), NOT_RECORDING, State::Idle),
            (RecordingOperation::StopToOutput, Err(&not_recording), NOT_RECORDING, State::Idle),
            (RecordingOperation::StopToOutput, Err(&timed_out), RECORDING, State::Recording),
            (RecordingOperation::Discard, Ok(()), NOT_RECORDING, State::Idle),
            (RecordingOperation::Discard, Err(&timed_out), NOT_RECORDING, State::Idle),
        ];
        for (operation, outcome, observation, expected) in cases {
            assert_eq!(
                State::after(operation, outcome, observation),
                expected,
                "{operation:?} finished with {outcome:?} while {observation:?}"
            );
        }
    }
}
