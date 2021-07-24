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
        if e != "asm" {
          return Err("file must have .asm extension");
        }
      } else {
        return Err("no file extension");
      }

      let filename_no_ext = Path::new(&input_filename)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap();

      let output_filename = String::from(filename_no_ext) + ".hack";

      Ok(Config {
        input_filename,
        output_filename,
      })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;
  let mut output = String::new();

  let mut symbol_table = HashMap::new();

  let mut line_number = 0;

  for line in input.split('\n') {
      let mut result = &line[..];

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
      
      if let Some(binary) = instruction.binary {
        line_number += 1;
        output = output + &binary + "\n";
      } else if let Some(label) = instruction.label {
        symbol_table.insert(
          label,
          line_number,
        );
      }
    }

  fs::write(config.output_filename, output)?;

  Ok(())
}

struct Instruction {
  _type: InstructionType,
  binary: Option<String>,
  label: Option<String>,
}

impl Instruction {
  fn new(s: &str) -> Instruction {
    let _type = InstructionType::get(s);

    let mut symbols = Vec::new();

    let mut label = None;
    let mut binary = None;

    if _type == InstructionType::A {
      let symbol = s.get(1..).unwrap();
      symbols.push(String::from(symbol));

      let value: i32 = symbol.parse().unwrap();
      binary = Some(String::from("0") + &format!("{:015b}", value));

    } else if _type == InstructionType::L {
      let symbol = s.get(1..s.len()-1).unwrap();
      label = Some(String::from(symbol));
    } else {
      let mut dest = "";
      let mut comp = "";
      let mut jmp = "";

      if let Some(d_index) = s.find("=") {
        dest = s.get(..d_index).unwrap();

        if let Some(j_index) = s.find(";") {
          comp = s.get(d_index+1..j_index).unwrap();
          jmp = s.get(j_index+1..).unwrap();
        } else {
          comp = s.get(d_index+1..).unwrap();
        }
      } else if let Some(j_index) = s.find(";") {
        comp = s.get(..j_index).unwrap();
        jmp = s.get(j_index+1..).unwrap();
      }

      let dest_binary = dest_to_binary(dest);
      let comp_binary = comp_to_binary(comp);
      let jmp_binary = jmp_to_binary(jmp);

      binary = Some(String::from("111") + &dest_binary + &comp_binary + &jmp_binary);

      symbols.push(String::from(dest));
      symbols.push(String::from(comp));
      symbols.push(String::from(jmp));
    }

    Instruction {
      _type,
      binary,
      label,
    }
  }
}

fn jmp_to_binary(jmp: &str) -> String {
  if jmp == "" {
    return String::from("000"); 
  }

  if jmp == "JGT" {
    return String::from("001"); 
  }

  if jmp == "JEQ" {
    return String::from("010"); 
  }

  if jmp == "JGE" {
    return String::from("011"); 
  }

  if jmp == "JLT" {
    return String::from("100"); 
  }

  if jmp == "JNE" {
    return String::from("101"); 
  }

  if jmp == "JLE" {
    return String::from("110"); 
  }

  return String::from("111"); 
}

fn comp_to_binary(comp: &str) -> String {
  if comp == "0" {
    return String::from("0101010");
  }

  if comp == "1" {
    return String::from("0111111");
  }

  if comp == "-1" {
    return String::from("0111010");
  }

  if comp == "D" {
    return String::from("0001100");
  }

  if comp == "A" {
    return String::from("0110000");
  }

  if comp == "M" {
    return String::from("1110000");
  }

  if comp == "!D" {
    return String::from("0001101");
  }

  if comp == "!A" {
    return String::from("0110001");
  }

  if comp == "!M" {
    return String::from("1110001");
  }

  if comp == "-D" {
    return String::from("0001111");
  }

  if comp == "-A" {
    return String::from("0110011");
  }

  if comp == "-M" {
    return String::from("1110011");
  }

  if comp == "D+1" {
    return String::from("0011111");
  }

  if comp == "A+1" {
    return String::from("0110111");
  }

  if comp == "M+1" {
    return String::from("1110111");
  }

  if comp == "D-1" {
    return String::from("0001110");
  }

  if comp == "A-1" {
    return String::from("0110010");
  }

  if comp == "M-1" {
    return String::from("1110010");
  }

  if comp == "D+A" {
    return String::from("0000010");
  }

  if comp == "D+M" {
    return String::from("1000010");
  }

  if comp == "D-A" {
    return String::from("0010011");
  }

  if comp == "D-M" {
    return String::from("1010011");
  }

  if comp == "A-D" {
    return String::from("0000111");
  }

  if comp == "M-D" {
    return String::from("1000111");
  }

  if comp == "D&A" {
    return String::from("0000000");
  }

  if comp == "D&M" {
    return String::from("1000000");
  }

  if comp == "D|A" {
    return String::from("0010101");
  }

  return String::from("1010101");
}

fn dest_to_binary(dest: &str) -> String {
  if dest == "" {
    return String::from("000");
  }

  if dest == "M" {
    return String::from("001");
  }

  if dest == "D" {
    return String::from("010");
  }

  if dest == "DM" {
    return String::from("011");
  }

  if dest == "A" {
    return String::from("100");
  }

  if dest == "AM" {
    return String::from("101");
  }

  if dest == "AD" {
    return String::from("110");
  }

  String::from("111")
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

