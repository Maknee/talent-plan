use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
struct Move {
    x: i32,
    y: i32
}

fn main() -> Result<()> {
    {
        let a = Move {x: 1, y: 2};
        // Serialize to Vec<u8>
        let v = serde_json::to_vec(&a)?;
        println!("j vec {v:#?}");

        // To string
        let s = std::str::from_utf8(&v)?;
        println!("j str {s:#?}");

        let v = ron::to_string(&a)?;
        println!("ron str {v:#?}");
        
    }

    println!("Hello, world!");
    Ok(())
}
