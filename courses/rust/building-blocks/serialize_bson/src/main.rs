use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::io::prelude::*;
use std::io::{BufWriter, BufReader};
use std::fs::File;
use bson::Document;

#[derive(Debug, Serialize, Deserialize)]
struct Move {
    x: i32,
    y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Moves {
    m: Vec<Move>
}


fn main() -> Result<()>{
    // Serialize many
    {
        let fname = "test.bson";
        {
            let f = File::create(&fname)?;
            let mut writer = BufWriter::new(f);
            for i in 0..5 {
                let m = Move{x: i, y:i};
                writer.write(&bson::to_vec(&m)?)?;
                println!("Write b {m:#?}");
            }
            writer.flush()?;
        }

        {
            let f = File::open(&fname)?;
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;

            let mut moves = Vec::new();
            let mut cursor = std::io::Cursor::new(buffer);
            while let Ok(doc) = Document::from_reader(&mut cursor) {
                if let Ok(v) = bson::from_document::<Move>(doc) {
                    moves.push(v);
                } else {
                    break;
                }
            }
            println!("Read {moves:#?}");
        }

        // // Writes
        // {
        //     let mut v = Moves{m: Vec::new()};
        //     for i in 0..5 {
        //         v.m.push(Move{x: i, y:i});
        //     }
        //     let f = File::create(&fname)?;
        //     let mut writer = BufWriter::new(f);
        //     let vv = bson::to_vec(&v)?;
        //     writer.write(&vv);
        //     writer.flush()?;
        //     println!("Write b {vv:#?}");
        // }

        // // Reads
        // {
        //     let f = File::open(&fname)?;
        //     let mut reader = BufReader::new(f);
        //     let mut v = Vec::with_capacity(8096);
        //     reader.read_to_end(&mut v)?;
        //     let moves: Moves = bson::from_slice(&v)?;
        //     println!("Read b {moves:#?}");
        // }
    }
    
    println!("Hello, world!");
    Ok(())
}
