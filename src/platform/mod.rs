#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
pub use windows::{lcu_auth, startup};

#[cfg(target_os = "macos")]
pub use macos::{lcu_auth, startup};
