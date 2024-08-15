use clap::{Parser, Subcommand};
use std::process;
use kvs::KvStore;
use anyhow::Result;

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
use std::env::current_dir;

fn main() -> Result<()> {
    let cli = Cli::parse();
    // let mut kvs = KvStore::open("data")?;
    let mut kvs = KvStore::open(current_dir()?)?;

    match cli.command {
        Command::Get { key } => {
            match kvs.get(key) {
                Ok(v) => {
                    if let Some(value) = v {
                        println!("{value}");
                    } else {
                        println!("Key not found");
                    }
                }
                Err(_) => {
                    println!("Key not found");
                }
            }
        },
        Command::Set { key, value } => {
            match kvs.set(key, value.clone()) {
                Ok(_) => {
                    // println!("{value}");
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        },
        Command::Rm { key } => {
            match kvs.remove(key.clone()) {
                Ok(_) => {
                    // println!("{key}");
                }
                Err(_) => {
                    println!("Key not found");
                    process::exit(1);
                }
            }
        },
    }
    Ok(())
}