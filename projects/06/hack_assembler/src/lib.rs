use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;

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
  let contents = fs::read_to_string(config.filename)?;

  Ok(())
}