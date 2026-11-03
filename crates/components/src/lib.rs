//! Dioxus components for Omriss, built on `omriss-ui`'s renderer-independent
//! view state (RFC-075).
//!
//! This crate holds everything that draws, and nothing that touches the
//! platform: no file system, no dialogs, no settings, no windowing. Those
//! belong to the host application (`omriss`). That split is what lets a second
//! host be written without untangling this crate.

pub mod draft_sync;
