//! Macro recording system for Grafeno Vetor.
//! Inspired by CorelDRAW's recording system with pause/resume support.

use grafeno_core::Command;

/// The state of a macro recording session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingState {
    /// Not recording.
    #[default]
    Stopped,
    /// Recording commands (active or paused).
    Recording,
    /// Recording is paused - user can review before resuming.
    Paused,
}

impl RecordingState {
    /// Returns true if we are actively recording (not paused).
    pub fn is_recording(&self) -> bool {
        matches!(self, RecordingState::Recording)
    }

    /// Returns true if recording is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self, RecordingState::Paused)
    }

    /// Start a new recording session.
    pub fn start() -> Self {
        RecordingState::Recording
    }

    /// Pause the current recording.
    pub fn pause(&self) -> Option<Self> {
        if self.is_recording() {
            Some(RecordingState::Paused)
        } else {
            None
        }
    }

    /// Resume a paused recording.
    pub fn resume(&self) -> Option<Self> {
        if self.is_paused() {
            Some(RecordingState::Recording)
        } else {
            None
        }
    }

    /// Stop the recording and return the collected commands.
    pub fn stop(self) -> Option<Vec<Command>> {
        match self {
            RecordingState::Recording | RecordingState::Paused => Some(Vec::new()),
            RecordingState::Stopped => None,
        }
    }

    /// Cancel the recording without saving.
    pub fn cancel(self) -> Self {
        RecordingState::Stopped
    }
}

/// A recording session that captures commands.
#[derive(Debug, Clone)]
pub struct RecordingSession {
    /// The current state.
    pub state: RecordingState,
    /// Commands collected so far.
    pub commands: Vec<Command>,
    /// When recording started (for auto-save).
    pub started_at: std::time::Instant,
}

impl RecordingSession {
    /// Start a new recording session.
    pub fn start() -> Self {
        Self {
            state: RecordingState::Recording,
            commands: Vec::new(),
            started_at: std::time::Instant::now(),
        }
    }

    /// Pause the recording.
    pub fn pause(&mut self) -> bool {
        if self.state.is_recording() {
            self.state = RecordingState::Paused;
            true
        } else {
            false
        }
    }

    /// Resume a paused recording.
    pub fn resume(&mut self) -> bool {
        if self.state.is_paused() {
            self.state = RecordingState::Recording;
            true
        } else {
            false
        }
    }

    /// Stop and return the commands.
    pub fn stop(&mut self) -> Vec<Command> {
        self.state = RecordingState::Stopped;
        std::mem::take(&mut self.commands)
    }

    /// Cancel without saving.
    pub fn cancel(&mut self) -> bool {
        if self.state != RecordingState::Stopped {
            self.state = RecordingState::Stopped;
            self.commands.clear();
            true
        } else {
            false
        }
    }

    /// Add a command to the recording.
    pub fn add(&mut self, cmd: Command) {
        if self.state.is_recording() {
            self.commands.push(cmd);
        }
    }
}
