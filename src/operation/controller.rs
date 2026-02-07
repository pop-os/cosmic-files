use atomic_float::AtomicF32;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicU16};
use tokio::sync::Notify;

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Default)]
#[repr(u16)]
pub enum ControllerState {
    Cancelled,
    Failed,
    Paused,
    #[default]
    Running,
}

#[derive(Debug, Default)]
struct ControllerInner {
    state: AtomicU16,
    progress: AtomicF32,
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
            inner: Arc::default(),
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
        self.inner.progress.load(atomic::Ordering::Relaxed)
    }

    pub fn set_progress(&self, progress: f32) {
        self.inner
            .progress
            .swap(progress, atomic::Ordering::Relaxed);
    }

    pub fn state(&self) -> ControllerState {
        ControllerState::try_from(self.inner.state.load(atomic::Ordering::Relaxed))
            .unwrap_or(ControllerState::Failed)
    }

    pub fn set_state(&self, state: ControllerState) {
        self.inner
            .state
            .store(state.into(), atomic::Ordering::Relaxed);
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

    /// Returns when the state is paused.
    ///
    /// Use this to pause futures.
    pub async fn until_paused(&self) {
        loop {
            if matches!(self.state(), ControllerState::Paused) {
                return;
            }

            self.inner.notify.notified().await;
        }
    }

    /// Returns when state is neither paused, cancelled, nor failed.
    ///
    /// Use this to resume futures.
    pub async fn until_unpaused(&self) {
        loop {
            if !matches!(
                self.state(),
                ControllerState::Paused | ControllerState::Cancelled | ControllerState::Failed
            ) {
                return;
            }

            self.inner.notify.notified().await;
        }
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
        if self.primary && !self.is_failed() {
            self.cancel();
        }
    }
}
