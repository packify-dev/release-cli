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
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Release { r#type } => {
            release::release(r#type);
        }
    }
}
