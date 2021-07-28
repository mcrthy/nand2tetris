use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use std::collections::HashMap;

pub struct Config {
  input_filename: String,
  output_filename: String,
}

impl Config {
  pub fn new (args: &[String]) -> Result<Config, &str> {
      if args.len() < 2 {
          return Err("not enough arguments");
      }

      let input_filename = args[1].clone();

      let extension = Path::new(&input_filename)
        .extension()
        .and_then(OsStr::to_str);

      if let Some(e) = extension {
        if e != "vm" {
          return Err("file must have .vm extension");
        }
      } else {
        return Err("no file extension");
      }

      let filename_no_ext = Path::new(&input_filename)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap();

      let output_filename = String::from(filename_no_ext) + ".asm";

      Ok(Config {
        input_filename,
        output_filename,
      })
  }
}

fn parse_instruction(line: &str) -> &str {
  let mut result = line;

  if let Some(index) = result.find("//") {
    if let Some (instruction) = result.get(..index) {
      result = instruction;
    }
  }

  if result.len() > 0 {
    result = result.trim();
  }

  result
}