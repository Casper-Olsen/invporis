mod cli;

use std::path::PathBuf;

use clap::Parser;
use directories::ProjectDirs;

fn main() {
    let args = cli::Args::parse();

    if args.total_value {
        println!("Getting total value ...");
    } else {
        eprintln!("Exiting!");
        return;
    }

    if let Some(dir) = get_data_directory() {
        println!("Using data directory: {}", dir.display());
    } else {
        eprintln!("Could not determine data directory");
    }
}

fn get_data_directory() -> Option<PathBuf> {
    ProjectDirs::from("com", "Casper-Olsen", "invporis")
        .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
}
