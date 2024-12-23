mod build;
mod release;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Release {
        #[clap(long, short)]
        r#type: String,
    },
    Build {
        #[clap(long, short)]
        tag: String,
    },
    Info {},
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Release { r#type } => {
            release::release(r#type);
        }
        Commands::Info {} => {
            println!(
                "channel: {}\nmajor: {}\nminor: {}\npatch: {}\nfull version string: {}",
                option_env!("RELEASE_CHANNEL").unwrap_or("unknown"),
                option_env!("RELEASE_MAJOR").unwrap_or("unknown"),
                option_env!("RELEASE_MINOR").unwrap_or("unknown"),
                option_env!("RELEASE_PATCH").unwrap_or("unknown"),
                option_env!("RELEASE_VERSION").unwrap_or("unknown")
            );
        }
        Commands::Build { tag } => {
            build::build(tag);
        }
    }
}
