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

      if let Some(index) = result.find("//") {
        if let Some (instruction) = result.get(..index) {
          result = instruction;
        }
      }

      if result.len() > 0 {
        result = result.trim();
      }

      if result == "" {
        continue
      }

      let instruction = Instruction::new(result);
    }
  }

  Ok(())
}

struct Instruction {
  _type: InstructionType,
}

impl Instruction {
  fn new(s: &str) -> Instruction {
    let _type = InstructionType::get(s);

    Instruction { _type }
  }
}

#[derive(PartialEq)]
enum InstructionType {
  A,
  C,
  L,
}

impl InstructionType {
  fn get(instruction: &str) -> InstructionType {
    let first = instruction.chars().next().unwrap();
  
    if first == '@' {
      return InstructionType::A;
    }
  
    if first == '(' {
      return InstructionType::L;
    }
  
    return InstructionType::C;
  }
}

