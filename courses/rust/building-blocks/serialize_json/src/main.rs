use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::io::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Serialize, Deserialize)]
struct Move
{
    x: i32,
    y: i32,
}

fn main() -> Result<()> {
    let a = Move{ x: 1, y: 2 };

    // Write
    {
        let f = File::create("test.json")?;
        let mut writer = BufWriter::new(f);
        serde_json::to_writer(&mut writer, &a)?;
        println!("Writing {a:#?}");
    }

    // Read
    {
        let f = File::open("test.json")?;
        let mut reader = BufReader::new(f);
        let b: Move = serde_json::from_reader(&mut reader)?;
        println!("Reading {b:#?}");
    }

    Ok(())
}
