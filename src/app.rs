use crate::dotfiles::Dotfiles;
use crate::util;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::borrow::Borrow;
use std::env;
use std::path::Path;
use url::Url;

#[cfg(windows)]
use crate::windows;

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
            dotfiles,
            dry_run,
            verbose,
        })
    }

    pub fn command_clone(&self, query: &str) -> Result<()> {
        let url = resolve_url(query)?;
        let dotdir = self.dotfiles.root_dir().to_string_lossy();
        util::wait_exec(
            "git",
            &["clone", url.as_str(), dotdir.borrow()],
            None,
            self.dry_run,
        )
        .map_err(Into::into)
    }

    pub fn command_root(&self) -> Result<()> {
        println!("{}", self.dotfiles.root_dir().display());
        Ok(())
    }

    pub fn command_check(&mut self) -> Result<i32> {
        self.dotfiles.read_entries();

        let mut num_unhealth = 0;
        for entry in self.dotfiles.entries() {
            if !entry.check(self.verbose).unwrap() {
                num_unhealth += 1;
            }
        }
        Ok(num_unhealth)
    }

    pub fn command_link(&mut self) -> Result<()> {
        self.dotfiles.read_entries();

        if !self.dry_run {
            check_symlink_privilege();
        }

        for entry in self.dotfiles.entries() {
            entry.mklink(self.dry_run, self.verbose).unwrap();
        }

        Ok(())
    }

    pub fn command_clean(&mut self) -> Result<()> {
        self.dotfiles.read_entries();

        for entry in self.dotfiles.entries() {
            entry.unlink(self.dry_run, self.verbose).unwrap();
        }

        Ok(())
    }
}

#[cfg(windows)]
fn check_symlink_privilege() {
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
        env::set_var("HOME", dirs::home_dir().unwrap());
    }

    let dotdir = env::var("DOT_DIR")
        .or_else(|_| util::expand_full("$HOME/.dotfiles"))
        .context("failed to determine dotdir")?;
    env::set_var("DOT_DIR", dotdir.as_str());
    env::set_var("dotdir", dotdir.as_str());

    Ok(dotdir)
}

fn resolve_url(s: &str) -> Result<Url> {
    let re_scheme = Regex::new(r"^([^:]+)://").unwrap();
    let re_scplike = Regex::new(r"^((?:[^@]+@)?)([^:]+):/?(.+)$").unwrap();

    if let Some(cap) = re_scheme.captures(s) {
        match cap.get(1).unwrap().as_str() {
            "http" | "https" | "ssh" | "git" | "file" => Url::parse(s).map_err(Into::into),
            scheme => Err(anyhow!("'{}' is invalid scheme", scheme)),
        }
    } else if let Some(cap) = re_scplike.captures(s) {
        let username = cap
            .get(1)
            .and_then(|s| {
                if s.as_str() != "" {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("git@");
        let host = cap.get(2).unwrap().as_str();
        let path = cap.get(3).unwrap().as_str();

        Url::parse(&format!("ssh://{}{}/{}.git", username, host, path)).map_err(Into::into)
    } else {
        let username = s
            .split_once('/')
            .map(|x| x.1)
            .context("'username' is unknown")?;
        let reponame = s.split_once('/').map(|x| x.1).unwrap_or("dotfiles");
        Url::parse(&format!("https://github.com/{}/{}.git", username, reponame)).map_err(Into::into)
    }
}
