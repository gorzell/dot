extern crate dot;

use crate::cli::Options;
use anyhow::Result;
use clap::{IntoApp, Parser};
use clap_complete::generator::generate;
use cli::Command;
use dot::App;
use std::io;

mod cli;

pub fn main() -> Result<()> {
    let opts = cli::Options::parse();
    let dry_run = opts.dry_run;
    let verbose = opts.verbose;
    let mut app = App::new(dry_run, verbose)?;

    match opts.command {
        // TODO: Figure out how we want to show the number of unhealthy items in the check.
        Command::Check => {
            app.command_check()?;
        }
        Command::Link => app.command_link()?,
        Command::Clean => app.command_clean()?,
        Command::Root => app.command_root()?,
        Command::Clone(args) => app.command_clone(&args.url)?,
        Command::Init(args) => {
            app.command_clone(&args.url)?;
            app.command_link()?
        }
        Command::Completion(args) => {
            generate(
                args.shell,
                &mut Options::into_app(),
                Options::into_app().get_name(),
                &mut io::stdout(),
            );
        }
    }
    Ok(())
}
