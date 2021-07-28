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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  Ok(())
}

enum Instruction {
  MoveInstruction(MoveInstruction),
  CalcInstruction(CalcInstruction),
}

enum MoveInstruction {
  Push(String),
  Pop(String),
}

impl MoveInstruction { 
  fn get (mv: &str, var: &str) -> MoveInstruction {
    if
  }
}

enum CalcInstruction {
  Lt,
  Eq,
  Gt,
  Add,
  Sub,
  And,
  Or,
  Not,
}

impl CalcInstruction {
  fn get(s: &str) -> CalcInstruction {
    match s {
      "lt"  => CalcInstruction::Lt,
      "eq"  => CalcInstruction::Eq,
      "gt"  => CalcInstruction::Gt,
      "add" => CalcInstruction::Add,
      "sub" => CalcInstruction::Sub,
      "and" => CalcInstruction::And,
      "or"  => CalcInstruction::Or,
      "not" => CalcInstruction::Not,
    }
  }
}

impl Instruction {
  fn get(s: &str) -> Instruction {
    if let Some(space_index) = s.find(" ") {
      let mv = s.get(..space_index).unwrap();
      let var = s.get(space_index+1..).unwrap();
      Instruction::MoveInstruction(MoveInstruction::get(mv, var))
    } else {
      Instruction::CalcInstruction(CalcInstruction::get(s))
    }
  }
}


