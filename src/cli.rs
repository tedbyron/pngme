use std::{env, fs, str::FromStr};

use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png, Error};

pub fn run() -> Result<(), Error> {
    let args = env::args().collect::<Box<_>>();
    match args.get(1).map(String::as_str) {
        Some("encode") => {
            if args.len() != 5 {
                Err("Invalid number of arguments: subcommand 'encode'".into())
            } else {
                encode(&args)
            }
        }
        Some("decode") => {
            if args.len() != 4 {
                Err("Invalid number of arguments: subcommand 'decode'".into())
            } else {
                decode(&args)
            }
        }
        Some("remove") => {
            if args.len() != 4 {
                Err("Invalid number of arguments: subcommand 'remove'".into())
            } else {
                remove(&args)
            }
        }
        Some("print") => {
            if args.len() != 3 {
                Err("Invalid number of arguments: subcommand 'print'".into())
            } else {
                print(&args)
            }
        }
        Some(s) => Err(format!("Invalid subcommand: {s}").into()),
        None => Err("Missing subcommand".into()),
    }
}

fn png_from_path(path: &str) -> Result<Png, Error> {
    let bytes = fs::read(path)?;
    Png::try_from(bytes.as_slice())
}

fn encode(args: &[String]) -> Result<(), Error> {
    let mut png = png_from_path(&args[2])?;
    png.append_chunk(Chunk::new(ChunkType::from_str(&args[3])?, &args[4]));
    fs::write(&args[2], png.bytes()).map_err(Error::from)
}

fn decode(args: &[String]) -> Result<(), Error> {
    let png = png_from_path(&args[2])?;
    match png.chunk_by_type(&args[3]) {
        Some(chunk) => Ok(println!("{}", chunk.data_as_string()?)),
        None => Err("Invalid chunk type".into()),
    }
}

fn remove(args: &[String]) -> Result<(), Error> {
    let mut png = png_from_path(&args[2])?;
    png.remove_chunk(&args[3])?;
    fs::write(&args[2], png.bytes()).map_err(Error::from)?;
    Ok(())
}

fn print(args: &[String]) -> Result<(), Error> {
    let png = png_from_path(&args[2])?;
    println!("{png}");
    Ok(())
}
