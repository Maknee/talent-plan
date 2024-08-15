use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "kvs", version, author, about = "A key-value store")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Get the value of a given key
    Get { 
        key: String 
    },
    /// Set the value of a key
    Set { 
        key: String,
        value: String,
    },
    /// Remove a key
    Rm { 
        key: String 
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Get { key } => {
            // println!("{}", key);  // For now, just echo the key
            eprintln!("unimplemented");
            process::exit(1);
        },
        Command::Set { key: _, value: _ } => {
            eprintln!("unimplemented");
            process::exit(1);
        },
        Command::Rm { key: _ } => {
            eprintln!("unimplemented");
            process::exit(1);
        },
    }
}