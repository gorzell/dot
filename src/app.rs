use std::borrow::Borrow;
use std::env;
use std::path::Path;
use dotfiles::Dotfiles;
use util;
use errors::Result;

#[cfg(windows)]
use windows;


pub struct App {
  dotfiles: Dotfiles,
  dry_run: bool,
  verbose: bool,
}

impl App {
  pub fn new(dry_run: bool, verbose: bool) -> Result<App> {
    let dotdir = init_envs()?;
    let dotfiles = Dotfiles::new(Path::new(&dotdir).to_path_buf());
    Ok(App {
      dotfiles: dotfiles,
      dry_run: dry_run,
      verbose: verbose,
    })
  }

  pub fn command_clone(&self, url: &str) -> Result<i32> {
    let dotdir = self.dotfiles.root_dir().to_string_lossy();
    util::wait_exec("git", &["clone", url, dotdir.borrow()], None, self.dry_run).map_err(Into::into)
  }

  pub fn command_root(&self) -> Result<i32> {
    println!("{}", self.dotfiles.root_dir().display());
    Ok(0)
  }

  pub fn command_check(&mut self) -> Result<i32> {
    self.dotfiles.read_entries();

    let mut num_unhealth = 0;
    for entry in self.dotfiles.entries() {
      if entry.check(self.verbose).unwrap() == false {
        num_unhealth += 1;
      }
    }
    Ok(num_unhealth)
  }

  pub fn command_link(&mut self) -> Result<i32> {
    self.dotfiles.read_entries();

    if !self.dry_run {
      check_symlink_privilege();
    }

    for entry in self.dotfiles.entries() {
      entry.mklink(self.dry_run, self.verbose).unwrap();
    }

    Ok(0)
  }

  pub fn command_clean(&mut self) -> Result<i32> {
    self.dotfiles.read_entries();

    for entry in self.dotfiles.entries() {
      entry.unlink(self.dry_run, self.verbose).unwrap();
    }

    Ok(0)
  }
}


#[cfg(windows)]
fn check_symlink_privilege() {
  use std::env;
  use std::process;
  use windows::ElevationType;

  match windows::get_elevation_type().unwrap() {
    ElevationType::Default => {
      match windows::enable_privilege("SeCreateSymbolicLinkPrivilege") {
        Ok(_) => (),
        Err(err) => panic!("failed to enable SeCreateSymbolicLinkPrivilege: {}", err),
      }
    }
    ElevationType::Limited => {
      panic!("should be elevate as an Administrator.");
    }
    ElevationType::Full => (),
  }
}

#[cfg(not(windows))]
#[inline]
pub fn check_symlink_privilege() {}


fn init_envs() -> Result<String> {
  if env::var("HOME").is_err() {
    env::set_var("HOME", env::home_dir().unwrap().to_str().unwrap());
  }

  let dotdir = env::var("DOT_DIR").or(util::expand_full("$HOME/.dotfiles"))
    .map_err(|_| "failed to determine dotdir".to_string())?;
  env::set_var("DOT_DIR", dotdir.as_str());
  env::set_var("dotdir", dotdir.as_str());

  Ok(dotdir)
}
