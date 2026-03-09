//! Platform abstraction layer for native (tokio) vs WASM (browser) runtimes.
//!
//! Provides unified types for:
//! - `MsgSender<T>` / `MsgReceiver<T>` — channel-like communication
//! - `PlatformRuntime` — async task spawning
//! - `create_channel()` — channel constructor
//! - `sleep_ms()` — async sleep
//! - `platform_spawn!` — macro dispatching to the right spawner

// ═══════════════════════════════════════════════════════════════════════
// Native (tokio + crossbeam)
// ═══════════════════════════════════════════════════════════════════════
#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub type MsgSender<T> = crossbeam_channel::Sender<T>;
    pub type MsgReceiver<T> = crossbeam_channel::Receiver<T>;

    pub fn create_channel<T>() -> (MsgSender<T>, MsgReceiver<T>) {
        crossbeam_channel::unbounded()
    }

    pub struct PlatformRuntime {
        pub rt: tokio::runtime::Runtime,
    }

    impl PlatformRuntime {
        pub fn new() -> Self {
            Self {
                rt: tokio::runtime::Runtime::new()
                    .expect("Failed to create tokio runtime"),
            }
        }
    }

    pub async fn sleep_ms(ms: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

// ═══════════════════════════════════════════════════════════════════════
// WASM (single-threaded, Rc<RefCell<VecDeque>>)
// ═══════════════════════════════════════════════════════════════════════
#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    /// A channel-like sender backed by a shared VecDeque (single-threaded).
    #[derive(Clone)]
    pub struct MsgSender<T> {
        inner: Rc<RefCell<VecDeque<T>>>,
    }

    impl<T> MsgSender<T> {
        pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
            self.inner.borrow_mut().push_back(msg);
            Ok(())
        }
    }

    /// Error type for compatibility with crossbeam-style API.
    #[derive(Debug)]
    pub struct SendError<T>(pub T);

    /// A channel-like receiver backed by a shared VecDeque (single-threaded).
    #[derive(Clone)]
    pub struct MsgReceiver<T> {
        inner: Rc<RefCell<VecDeque<T>>>,
    }

    /// Error type matching crossbeam's `TryRecvError`.
    #[derive(Debug)]
    pub enum TryRecvError {
        Empty,
    }

    impl<T> MsgReceiver<T> {
        pub fn try_recv(&self) -> Result<T, TryRecvError> {
            self.inner
                .borrow_mut()
                .pop_front()
                .ok_or(TryRecvError::Empty)
        }
    }

    pub fn create_channel<T>() -> (MsgSender<T>, MsgReceiver<T>) {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        (
            MsgSender { inner: queue.clone() },
            MsgReceiver { inner: queue },
        )
    }

    /// No-op runtime for WASM — tasks are spawned via `wasm_bindgen_futures::spawn_local`.
    pub struct PlatformRuntime;

    impl PlatformRuntime {
        pub fn new() -> Self {
            Self
        }
    }

    pub async fn sleep_ms(ms: u64) {
        gloo_timers::future::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// ═══════════════════════════════════════════════════════════════════════
// Spawn macro — dispatches to tokio or wasm_bindgen_futures
// ═══════════════════════════════════════════════════════════════════════

/// Spawn an async task on the appropriate runtime.
///
/// On native: `$runtime.rt.spawn($fut)` — requires `$fut: Send + 'static`
/// On WASM:   `wasm_bindgen_futures::spawn_local($fut)` — no Send required
///
/// The `$runtime` argument is the `PlatformRuntime` (ignored on WASM).
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! platform_spawn {
    ($runtime:expr, $fut:expr) => {
        $runtime.rt.spawn($fut)
    };
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! platform_spawn {
    ($runtime:expr, $fut:expr) => {{
        let _ = &$runtime; // suppress unused warning
        wasm_bindgen_futures::spawn_local($fut)
    }};
}
