use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePhase {
    Idle,
    Loading,
    Ready,
    Stale,
    Error,
}

impl ResourcePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceState<T> {
    phase: ResourcePhase,
    last_good: Option<T>,
    loading_started_at: Option<Instant>,
    last_error: Option<String>,
}

impl<T> ResourceState<T> {
    pub fn idle() -> Self {
        Self {
            phase: ResourcePhase::Idle,
            last_good: None,
            loading_started_at: None,
            last_error: None,
        }
    }

    pub fn ready(payload: T) -> Self {
        Self {
            phase: ResourcePhase::Ready,
            last_good: Some(payload),
            loading_started_at: None,
            last_error: None,
        }
    }

    pub fn begin_refresh(&mut self, started_at: Instant) {
        self.phase = ResourcePhase::Loading;
        self.loading_started_at = Some(started_at);
    }

    pub fn begin_refresh_now(&mut self) {
        self.begin_refresh(Instant::now());
    }

    pub fn finish_ok(&mut self, payload: T) {
        self.phase = ResourcePhase::Ready;
        self.last_good = Some(payload);
        self.loading_started_at = None;
        self.last_error = None;
    }

    pub fn finish_error(&mut self, error: impl Into<String>) {
        self.phase = ResourcePhase::Error;
        self.loading_started_at = None;
        self.last_error = Some(error.into());
    }

    pub fn expire_if_overdue(&mut self, timeout: Duration, detail: impl Into<String>) -> bool {
        if self.phase != ResourcePhase::Loading {
            return false;
        }
        let Some(started_at) = self.loading_started_at else {
            return false;
        };
        if started_at.elapsed() < timeout {
            return false;
        }

        self.phase = ResourcePhase::Stale;
        self.loading_started_at = None;
        self.last_error = Some(detail.into());
        true
    }

    pub fn phase(&self) -> ResourcePhase {
        self.phase
    }

    pub fn is_loading(&self) -> bool {
        self.phase == ResourcePhase::Loading
    }

    pub fn last_good(&self) -> Option<&T> {
        self.last_good.as_ref()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}
