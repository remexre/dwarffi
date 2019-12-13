use anyhow::{Context, Result};
use dwarffi::dwarf::get_items;
use std::{fs::read, path::PathBuf};

/// Simple code streaming server with asciinema and xterm.js.
#[derive(Debug, structopt::StructOpt)]
struct Args {
    /// Increases the verbosity of logging.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: usize,

    /// The .so to generate bindings to.
    pub file: PathBuf,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    let mut logger = stderrlog::new();
    if args.verbosity < 3 {
        logger.module(module_path!()).verbosity(2 + args.verbosity);
    } else {
        logger.verbosity(args.verbosity);
    }
    logger.init().unwrap();

    let file = read(&args.file).context("Failed to read file")?;
    let items = get_items(&file).context("Failed to get items from file")?;
    dwarffi::python::make_ffi(&args.file, &items)?;
    Ok(())
}
