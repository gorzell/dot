extern crate dot;

use crate::cli::Options;
use clap::{IntoApp, Parser};
use clap_generate::generate;
use cli::Command;
use dot::App;
use std::io;

mod cli;

pub fn main() {
    match run() {
        Ok(retcode) => std::process::exit(retcode),
        Err(err) => panic!("unknown error: {}", err),
    }
}

pub fn run() -> dot::Result<i32> {
    let opts = cli::Options::parse();
    let dry_run = opts.dry_run;
    let verbose = opts.verbose;
    let mut app = App::new(dry_run, verbose)?;

    match opts.command {
        Command::Check => app.command_check(),
        Command::Link => app.command_link(),
        Command::Clean => app.command_clean(),
        Command::Root => app.command_root(),
        Command::Clone(args) => app.command_clone(&args.url),
        Command::Init(args) => {
            let ret = app.command_clone(&args.url)?;
            if ret != 0 {
                return Ok(ret);
            }
            app.command_link()
        }
        Command::Completion(args) => {
            generate(
                args.shell,
                &mut Options::into_app(),
                Options::into_app().get_bin_name().unwrap(),
                &mut io::stdout(),
            );
            Ok(0)
        }
    }
}
