use crate::fl;

use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Copy, Debug)]
pub enum ControllerState {
    Cancelled,
    Paused,
    Running,
}

#[derive(Debug)]
struct ControllerInner {
    state: Mutex<ControllerState>,
    progress: Mutex<f32>,
    condvar: Condvar,
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
                condvar: Condvar::new(),
            }),
        }
    }
}

impl Controller {
    pub fn check(&self) -> Result<(), String> {
        let mut state = self.inner.state.lock().unwrap();
        loop {
            match *state {
                ControllerState::Cancelled => return Err(fl!("cancelled")),
                ControllerState::Paused => {
                    state = self.inner.condvar.wait(state).unwrap();
                }
                ControllerState::Running => return Ok(()),
            }
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
        self.inner.condvar.notify_all();
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.state(), ControllerState::Cancelled)
    }

    pub fn cancel(&self) {
        self.set_state(ControllerState::Cancelled);
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state(), ControllerState::Paused)
    }

    pub fn pause(&self) {
        self.set_state(ControllerState::Paused);
    }

    pub fn unpause(&self) {
        //TODO: ensure this does not override Cancel?
        self.set_state(ControllerState::Running);
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
        // Cancel operations if primary controller is dropped
        if self.primary {
            self.cancel();
        }
    }
}
