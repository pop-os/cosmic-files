use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerState {
    Cancelled,
    Failed,
    Paused,
    Running,
}

#[derive(Debug)]
struct ControllerInner {
    state: Mutex<ControllerState>,
    progress: Mutex<f32>,
    notify: Notify,
}

#[derive(Debug)]
pub struct Controller {
    primary: bool,
    inner: Arc<ControllerInner>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            primary: true,
            inner: Arc::new(ControllerInner {
                state: Mutex::new(ControllerState::Running),
                progress: Mutex::new(0.0),
                notify: Notify::new(),
            }),
        }
    }
}

impl Controller {
    pub async fn check(&self) -> Result<(), ControllerState> {
        loop {
            match self.state() {
                ControllerState::Cancelled => return Err(ControllerState::Cancelled),
                ControllerState::Failed => return Err(ControllerState::Failed),
                ControllerState::Paused => (),
                ControllerState::Running => return Ok(()),
            }

            self.inner.notify.notified().await;
        }
    }

    pub fn progress(&self) -> f32 {
        *self.inner.progress.lock().unwrap()
    }

    pub fn set_progress(&self, progress: f32) {
        *self.inner.progress.lock().unwrap() = progress;
    }

    pub fn state(&self) -> ControllerState {
        *self.inner.state.lock().unwrap()
    }

    pub fn set_state(&self, state: ControllerState) {
        *self.inner.state.lock().unwrap() = state;
        self.inner.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.state(), ControllerState::Cancelled)
    }

    pub fn cancel(&self) {
        self.set_state(ControllerState::Cancelled);
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state(), ControllerState::Failed)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state(), ControllerState::Paused)
    }

    pub fn pause(&self) {
        self.set_state(ControllerState::Paused);
    }

    pub fn unpause(&self) {
        if !self.is_cancelled() | !self.is_failed() {
            self.set_state(ControllerState::Running);
        }
    }
}

impl Clone for Controller {
    fn clone(&self) -> Self {
        Self {
            primary: false,
            inner: self.inner.clone(),
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        // Cancel operations if primary controller is dropped and controller is still running
        if self.primary && self.state() != ControllerState::Failed {
            self.cancel();
        }
    }
}
