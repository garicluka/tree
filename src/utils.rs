use crate::{cli, types::Result};
use clap::Parser;
use std::{fs, path::PathBuf};

pub fn get_current_path() -> Result<PathBuf> {
    let args = cli::Args::parse().path;
    match args {
        Some(p) => Ok(fs::canonicalize(p)?),
        None => Ok(fs::canonicalize(".")?),
    }
}
