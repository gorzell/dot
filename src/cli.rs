use clap::{Parser, Subcommand};
use clap_complete::shells::Shell;

#[derive(Parser, Debug)]
#[clap(version, about)]
#[clap(propagate_version = true)]
#[clap(subcommand_required = true)]
#[clap(arg_required_else_help = true)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(short, long, help = "Print actions to be taken")]
    pub dry_run: bool,
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check the files are correctly linked to the right places
    Check,
    /// Create all of the symbolic links into home directory
    Link,
    /// Remote all of registered links from home directory
    Clean,
    /// Show the location of dotfiles repository and exit
    Root,
    /// Clone dotfiles repository from remote
    Clone(CloneOptions),
    /// Clone dotfiles repository from remote & make links
    Init(CloneOptions),
    /// Generate completion scripts
    Completion(CompletionOptions),
}

#[derive(Parser, Debug)]
pub struct CloneOptions {
    #[clap(help = "Git URL.")]
    pub url: String,
}

#[derive(Parser, Debug)]
pub struct CompletionOptions {
    #[clap(value_enum, help = "Shell name.")]
    pub shell: Shell,
}
