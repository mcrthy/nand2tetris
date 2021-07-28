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

fn preload(mut symbol_table: HashMap<String, i32>) -> HashMap<String, i32> {
  symbol_table.insert(String::from("R0"), 0);
  symbol_table.insert(String::from("R1"), 1);
  symbol_table.insert(String::from("R2"), 2);
  symbol_table.insert(String::from("R3"), 3);
  symbol_table.insert(String::from("R4"), 4);
  symbol_table.insert(String::from("R5"), 5);
  symbol_table.insert(String::from("R6"), 6);
  symbol_table.insert(String::from("R7"), 7);
  symbol_table.insert(String::from("R8"), 8);
  symbol_table.insert(String::from("R9"), 9);
  symbol_table.insert(String::from("R10"), 10);
  symbol_table.insert(String::from("R11"), 11);
  symbol_table.insert(String::from("R12"), 12);
  symbol_table.insert(String::from("R13"), 13);
  symbol_table.insert(String::from("R14"), 14);
  symbol_table.insert(String::from("R15"), 15);
  symbol_table.insert(String::from("SP"), 0);
  symbol_table.insert(String::from("LCL"), 1);
  symbol_table.insert(String::from("ARG"), 2);
  symbol_table.insert(String::from("THIS"), 3);
  symbol_table.insert(String::from("THAT"), 4);
  symbol_table.insert(String::from("SCREEN"), 16384);
  symbol_table.insert(String::from("KBD"), 24576);

  symbol_table
}

fn add_labels(mut symbol_table: HashMap<String, i32>, instructions: &Vec<Instruction>) -> HashMap<String, i32> {
  let mut line_number = 0;

  for instruction in instructions.iter() {
    match instruction {
      Instruction::LInstruction(label) => {
        symbol_table.insert(
          label.clone(),
          line_number
        );
      },
      _ => line_number += 1,
    }
  }
  symbol_table
}

fn generate_output(mut symbol_table: HashMap<String, i32>, instructions: &Vec<Instruction>) -> String {
  let mut output= String::new();
  let mut curr_ram_loc = 16;

  for instruction in instructions.iter() {
    match instruction {
      Instruction::AInstruction(a_instruction) => {
        match a_instruction {
          AInstruction::Num(binary) => {
            output.push_str(&format!("{}\n", binary));
          },
          AInstruction::Var(name) => {
            if let Some(value) = symbol_table.get(name) {
              output.push_str(&format!("{:016b}\n", value));
            } else {
              symbol_table.insert(
                name.clone(),
                curr_ram_loc,
              );

              output.push_str(&format!("{:016b}\n", curr_ram_loc));

              curr_ram_loc += 1;
            }
          },
        }
      },
      Instruction::CInstruction(binary) => {
        output.push_str(&format!("{}\n", binary));
      },
      _ => continue,
    }
  }
  output
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  let symbol_table: HashMap<String, i32> = HashMap::new();
  let symbol_table = preload(symbol_table);

  let instructions = get_instructions(input);

  let symbol_table = add_labels(symbol_table, &instructions);

  let output = generate_output(symbol_table, &instructions);

  fs::write(config.output_filename, output)?;
  Ok(())
}

// Get the instructions out of a file
fn get_instructions(input: String) -> Vec<Instruction> {
  let stripped_file = strip_file(input);

  let mut result = Vec::new();

  for line in stripped_file {
    result.push(Instruction::get(&line));
  }

  result
}

// Separates a string by newline, strips comments and whitespace from each line
fn strip_file(input: String) -> Vec<String> {
  let mut result = Vec::new();

  for line in input.split('\n') {
    if let Some(instruction) = strip_line(line) {
      result.push(String::from(instruction));
    }
  }

  result
}

// Strips comments and whitespace from a line.
// If there are no chars left after stripping, return None.
fn strip_line(line: &str) -> Option<&str> {
  let mut result = line;

  if let Some(index) = result.find("//") {
    if let Some (instruction) = result.get(..index) {
      result = instruction;
    }
  }

  if result.len() > 0 {
    result = result.trim();
  }

  if result == "" {
    None
  } else {
    Some(result)
  }
}

#[derive(PartialEq)]
enum Instruction {
  AInstruction(AInstruction),
  LInstruction(String),
  CInstruction(String),
}

#[derive(PartialEq)]
enum AInstruction {
  Num(String),
  Var(String),
}

impl AInstruction {
  fn get(s: &str) -> AInstruction {
    match s.parse::<i32>() {
      Ok(num) => {
        let binary = format!("{:016b}", num);
        println!("binary: {}", binary);
        AInstruction::Num(binary)
      },
      _ => {
        let name = String::from(s);
        AInstruction::Var(name)
      }
    }
  }
}

impl Instruction {
  fn get(s: &str) -> Instruction {
    let head = s.get(0..1).unwrap();
    let tail = s.get(1..).unwrap();
    let body = s.get(1..s.len()-1).unwrap();

    if head == "@" {
      println!("a instruction: {}", s);
      println!("tail: {}", tail);
      Instruction::AInstruction(AInstruction::get(tail))
    } else if head == "(" {
      Instruction::LInstruction(String::from(body))
    } else {
      let binary = construct_comp_binary(s);
      Instruction::CInstruction(binary)
    }
  }
}

fn construct_comp_binary(s: &str) -> String {
  let dest = parse_dest(s);
  let jmp = parse_jmp(s);
  let comp = parse_comp(s);

  let dest_binary = dest_to_binary(dest);
  let comp_binary = comp_to_binary(comp);
  let jmp_binary = jmp_to_binary(jmp);

  String::from("111") + &comp_binary + &dest_binary + &jmp_binary
}

fn parse_comp(s: &str) -> &str {
  let result: Option<&str>;

  if let Some(d_index) = s.find("=") {
    if let Some(j_index) = s.find(";") {
      result = s.get(d_index+1..j_index);
    } else {
      result = s.get(d_index+1..);
    }
  } else if let Some(j_index) = s.find(";") {
    result = s.get(..j_index);
  } else {
    result = Some(s);
  }

  if let Some(r) = result {
    r
  } else {
    ""
  }
}

fn parse_dest(s: &str) -> &str {
  let result: &str;

  match s.find("=") {
    Some(d_index) => {
      match s.get(..d_index) {
        Some(d) => result = d,
        None    => result = "",
      }
    },
    None => result = "",
  }

  result
}

fn parse_jmp(s: &str) -> &str {
  let result: &str;

  match s.find(";") {
    Some(j_index) => {
      match s.get(j_index+1..) {
        Some(j) => result = j,
        None    => result = "",
      }
    }
    None => result = "",
  }
  
  result
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
  } else if dest == "DM" || dest == "MD" {
    result = "011";
  } else if dest == "A" {
    result = "100";
  } else if dest == "AM" || dest == "MA" {
    result = "101";
  } else if dest == "AD" || dest == "DA" {
    result = "110";
  } else {  // dest == "ADM"
    result = "111";
  }

  result
}