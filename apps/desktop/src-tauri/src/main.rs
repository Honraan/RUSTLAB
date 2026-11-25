#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(any(windows, test))]
mod course;
#[cfg(windows)]
mod desktop;
#[cfg(any(windows, test))]
mod tasks;
#[cfg(any(windows, test))]
mod workspace;

#[cfg(windows)]
fn main() {
    desktop::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("RustLab Desktop é distribuído para Windows.");
}
