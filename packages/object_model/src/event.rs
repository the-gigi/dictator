//! The callback shape that `platform` fires and `engine` reacts to.
//!
//! This is the *entire* event vocabulary shared between packages: a trait, no
//! enum. `platform` calls these methods; how the work then reaches a worker
//! thread is `engine`'s private business.
//!
//! # Why no `Event` enum here
//!
//! `engine` does need some message type internally, because none of these
//! callbacks can do real work on the thread that fires them (see below). But
//! that type never appears in a signature any other package can see, so by the
//! rule that this package holds only what more than one package needs, it stays
//! private to `engine`.
//!
//! # Who fires these, and why the implementor must be trivial
//!
//! | Callback            | Fired from                                  |
//! |---------------------|---------------------------------------------|
//! | `on_start`/`on_stop`| the main thread, inside the CFRunLoop       |
//! | `on_capture_error`  | cpal's error callback                       |
//!
//! So `Engine` must never implement this trait directly. Doing real work in
//! `on_start` would block the run loop and drop key events. The only
//! implementor forwards across a thread boundary and returns immediately.
//!
//! # Why audio isn't here
//!
//! Samples never pass through this trait. cpal's data callback runs on a
//! real-time thread that must not allocate or block, so it pushes straight into
//! an `rtrb` ring buffer and the engine's worker drains the other end. This
//! trait carries only control events, which are rare and can allocate freely.
//!
//! # Why `&self` rather than `&mut self`
//!
//! The main thread and cpal's error callback hold the sink concurrently, so the
//! methods take `&self` and a single `Arc<dyn DictatorEvents>` is shared by both
//! callers. `Sender<T>` is `Sync` in current Rust, so the forwarding
//! implementation needs no lock.
//!
//! Holds: `DictatorEvents`.

pub trait DictatorEvents: Send + Sync {
    fn on_start(&self);
    fn on_stop(&self);
    fn on_capture_error(&self, message: &str);
}
