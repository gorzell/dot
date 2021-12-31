extern crate ansi_term;
extern crate regex;
extern crate shellexpand;
extern crate toml;
extern crate url;

#[cfg(windows)]
extern crate advapi32;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

pub mod app;
mod dotfiles;
mod entry;
pub mod util;
#[cfg(windows)]
mod windows;

pub use crate::app::App;
