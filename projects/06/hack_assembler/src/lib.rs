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
      let parsed = parse_instruction(line);

      if parsed == "" {
        continue
      }

      let instruction = Instruction::new(parsed);
      
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


struct Instruction {
  _type: InstructionType,
  binary: Option<String>,
  label: Option<String>,
}

impl Instruction {
  fn new(s: &str) -> Instruction {
    let _type = InstructionType::get(s);

    let mut label = None;
    let mut binary = None;

    if _type == InstructionType::A {
      let symbol = s.get(1..).unwrap();

      if let Ok(num) = symbol.parse::<i32>() {
        binary = Some(format!("{:016b}", num));
      } else {
        label = Some(String::from(symbol));
      }
      
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
    }

    Instruction {
      _type,
      binary,
      label,
    }
  }
}

fn jmp_to_binary(jmp: &str) -> &str {
  let result: &str;

  if jmp == "" {
    result = "000"; 
  } else if jmp == "JGT" {
    result = "001"; 
  } else if jmp == "JEQ" {
    result = "010"; 
  } else if jmp == "JGE" {
    result = "011"; 
  } else if jmp == "JLT" {
    result = "100"; 
  } else if jmp == "JNE" {
    result = "101"; 
  } else if jmp == "JLE" {
    result = "110"; 
  } else { // jmp == "JMP"
    result = "111";
  }

  result
}

fn comp_to_binary(comp: &str) -> &str {
  let result: &str;

  if comp == "0" {
    result = "0101010";
  } else if comp == "1" {
    result = "0111111";
  } else if comp == "-1" {
    result = "0111010";
  } else if comp == "D" {
    result = "0001100";
  } else if comp == "A" {
    result = "0110000";
  } else if comp == "M" {
    result = "1110000";
  } else if comp == "!D" {
    result = "0001101";
  } else if comp == "!A" {
    result = "0110001";
  } else if comp == "!M" {
    result = "1110001";
  } else if comp == "-D" {
    result = "0001111";
  } else if comp == "-A" {
    result = "0110011";
  } else if comp == "-M" {
    result = "1110011";
  } else if comp == "D+1" {
    result = "0011111";
  } else if comp == "A+1" {
    result = "0110111";
  } else if comp == "M+1" {
    result = "1110111";
  } else if comp == "D-1" {
    result = "0001110";
  } else if comp == "A-1" {
    result = "0110010";
  } else if comp == "M-1" {
    result = "1110010";
  } else if comp == "D+A" {
    result = "0000010";
  } else if comp == "D+M" {
    result = "1000010";
  } else if comp == "D-A" {
    result = "0010011";
  } else if comp == "D-M" {
    result = "1010011";
  } else if comp == "A-D" {
    result = "0000111";
  } else if comp == "M-D" {
    result = "1000111";
  } else if comp == "D&A" {
    result = "0000000";
  } else if comp == "D&M" {
    result = "1000000";
  } else if comp == "D|A" {
    result = "0010101";
  } else {  // comp == "D|M"
    result = "1010101";
  }

  result
}

fn dest_to_binary(dest: &str) -> &str {
  let result: &str;

  if dest == "" {
    result = "000";
  } else if dest == "M" {
    result = "001";
  } else if dest == "D" {
    result = "010";
  } else if dest == "DM" {
    result = "011";
  } else if dest == "A" {
    result = "100";
  } else if dest == "AM" {
    result = "101";
  } else if dest == "AD" {
    result = "110";
  } else {  // dest == "ADM"
    result = "111";
  }

  result
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
    let result: InstructionType;
  
    if first == '@' {
      result = InstructionType::A;
    } else if first == '(' {
      result = InstructionType::L;
    } else {
      result = InstructionType::C;
    }

    result
  }
}

