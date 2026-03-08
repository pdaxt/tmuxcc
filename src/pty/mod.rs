//! PTY management — spawn and own terminal processes directly.
//!
//! Replaces tmux as the process host. Each "pane" is a real PTY
//! that we read/write to directly, giving us:
//!   - Zero-latency input (no tmux send-keys round-trip)
//!   - Real-time output streaming (no polling)
//!   - Native paste support (bracketed paste passthrough)
//!   - Full control over the process lifecycle

mod manager;
mod session;

pub use manager::PtyManager;
pub use session::{PtySession, PtySessionHandle, SessionEvent};
