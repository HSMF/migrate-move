use std::{error::Error, fmt::Display, path::PathBuf};

use clap::{Parser, Subcommand};
use migrate_move::{pattern::Pattern, Entries};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    /// the directory in which to search
    path: PathBuf,

    #[clap(short = 'u', long = "up")]
    /// pattern to use for up migration file names, index should be denoted with %d, name with %s
    pattern_up: Option<String>,
    #[clap(short = 'd', long = "down")]
    /// pattern to use for down migration file names, index should be denoted with %d, name with %s
    pattern_down: Option<String>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// list the current migrations
    List {
        #[clap(short, long)]
        /// only show up migrations
        only_up: bool,
    },

    /// move the specified migration up by one, i.e. to be run earlier
    Up {
        /// the index of the migration to move
        index: usize,
        #[clap(short, long)]
        /// by how much to move
        by: Option<usize>,
    },

    /// move the specified migration down by one, i.e. to be run later
    Down {
        /// the index of the migration to move
        index: usize,
        #[clap(short, long)]
        /// by how much to move
        by: Option<usize>,
    },
}

#[derive(Debug)]
struct InputError(&'static str);

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InputError {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let pattern_up = args
        .pattern_up
        .unwrap_or_else(|| "%d_%s.up.sql".to_string());

    let pattern_up = Pattern::parse(&pattern_up).map_err(|_| InputError("invalid up pattern"))?;

    let pattern_down = args
        .pattern_down
        .unwrap_or_else(|| "%d_%s.down.sql".to_string());
    let pattern_down =
        Pattern::parse(&pattern_down).map_err(|_| InputError("invalid down pattern"))?;

    let entries = Entries::from_dir(&args.path, pattern_up.clone(), pattern_down.clone())
        .expect("could not read directory");

    match args.command {
        Commands::List { only_up } => {
            for (i, entry) in entries.into_iter().enumerate() {
                println!(
                    "{i:>3}{}: {}",
                    if only_up { "" } else { " entry UP  " },
                    entry.migration.to_string(&pattern_up)
                );
                if !only_up {
                    println!(
                        "{i} entry DOWN: {}",
                        entry.migration.to_string(&pattern_down)
                    );
                }
            }
        }

        Commands::Up { index, by } => {
            let mut entries = entries;
            let by = by.unwrap_or(1);
            for i in 0..by {
                entries.move_up(index - i).expect("out of bounds");
            }
            entries.write_back().expect("could not write back");
        }

        Commands::Down { index, by } => {
            let mut entries = entries;
            let by = by.unwrap_or(1);
            for i in 0..by {
                entries.move_down(index + i).expect("out of bounds");
            }
            entries.write_back().expect("could not write back");
        }
    }

    Ok(())
}
