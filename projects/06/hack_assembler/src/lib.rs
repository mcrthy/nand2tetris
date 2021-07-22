use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use std::io::{self, BufRead};

pub struct Config {
  pub filename: String,
}

impl Config {
  pub fn new (args: &[String]) -> Result<Config, &str> {
      if args.len() < 2 {
          return Err("not enough arguments");
      }

      let filename = args[1].clone();

      let extension = Path::new(&filename)
        .extension()
        .and_then(OsStr::to_str);

      if let Some(e) = extension {
        if e != "asm" {
          return Err("file must have .asm extension");
        }
      } else {
        return Err("no file extension");
      }
  
      Ok(Config { filename })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let file = File::open(config.filename)?;

  let lines = io::BufReader::new(file).lines();

  for line in lines {
    if let Ok(ln) = line {

      let mut result = &ln[..];

      if let Some(index) = ln.find("//") {
        if let Some (instruction) = ln.get(..index) {
          result = instruction;
        }
      }

      if result.len() > 0 {
        result = result.trim();
      }

      if result == "" {
        continue
      }
    }
  }

  Ok(())
}