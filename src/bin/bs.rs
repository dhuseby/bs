//extern crate bs;
extern crate structopt;
extern crate sodiumoxide;

use bs::hash;
//use std::fs::File;
//use std::io::{self, Write};
//use std::path::{Path, PathBuf};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "BetterSign", 
    version = "0.1",
    author = "David Huseby <dhuseby@linuxfoundation.org>",
    about = "BetterSign signing tool",
)]
struct Opt {
    /// verbose output
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,

    /// the file descriptor number to use for machine parseable status
    #[structopt(long = "status-fd")]
    fd: Option<u32>,

    /// the subcommand operation
    #[structopt(subcommand)]
    cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {

    #[structopt(name = "hash")]
    /// Hash the given file(s) or data.
    Hash {
        /// List of paths to hash or '-' if signing data passed through stdin.
        #[structopt(name = "FILES", parse(from_os_str))]
        paths: Vec<PathBuf>,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // initialize sodiumoxide
    sodiumoxide::init().unwrap();

    // parse the command line flags
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Hash { paths } => {
            let _ = hash::hash(paths)?;
        }
    }

    Ok(())
}
