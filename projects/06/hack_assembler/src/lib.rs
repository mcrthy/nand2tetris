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

  let symbol_table: HashMap<String, i32> = HashMap::new();
  let symbol_table = preload(symbol_table);

  let instructions = get_instructions(input);

  let symbol_table = add_labels(symbol_table, &instructions);

  let output = generate_output(symbol_table, &instructions);

  fs::write(config.output_filename, output)?;
  Ok(())
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

// Get the instructions out of a file
fn get_instructions(input: String) -> Vec<Instruction> {
  let mut result = Vec::new();

  for line in input.split('\n') {
    let instruction = strip_line(line);
    if !instruction.is_empty() {
      result.push(Instruction::get(instruction));
  }
    }
    
  result
}

// Strips comments and whitespace from a line.
fn strip_line(line: &str) -> &str {
  match line.find("//") {
    Some(index) => line.get(..index).unwrap().trim(),
    None        => line.trim()
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
    match s.chars().next().unwrap() {
      '@' => {
        let result = s.get(1..).unwrap();
        Instruction::AInstruction(AInstruction::get(result))
      },
      '(' => {
        let result = s.get(1..s.len()-1).unwrap();
        Instruction::LInstruction(String::from(result))
      },
      _   => {
        let result = construct_comp_binary(s);
        Instruction::CInstruction(result)
      }
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
  match s.find("=") {
    Some(d_index) => {
      match s.find(";") {
        Some(j_index) => s.get(d_index+1..j_index).unwrap(),
        None          => s.get(d_index+1..).unwrap(),
      }
    },
    None => {
      match s.find(";") {
        Some(j_index) => s.get(..j_index).unwrap(),
        None          => s
      }
    }
  }
}

fn parse_dest(s: &str) -> &str {
  match s.find("=") {
    Some(d_index) => {
      match s.get(..d_index) {
        Some(d) => d,
        None    => "",
      }
    },
    None => "",
  }
}

fn parse_jmp(s: &str) -> &str {
  match s.find(";") {
    Some(j_index) => {
      match s.get(j_index+1..) {
        Some(j) => j,
        None    => ""
      }
    },
    None => ""
  }
}

fn jmp_to_binary(jmp: &str) -> &str {
  match jmp {
    ""    => "000",
    "JGT" => "001", 
    "JEQ" => "010", 
    "JGE" => "011", 
    "JLT" => "100", 
    "JNE" => "101", 
    "JLE" => "110", 
    _     =>  "111", // jmp == "JMP"
  }
}

fn comp_to_binary(comp: &str) -> &str {
  match comp {
    "0"   => "0101010",
    "1"   => "0111111",
    "-1"  => "0111010",
    "D"   => "0001100",
    "A"   => "0110000",
    "M"   => "1110000",
    "!D"  => "0001101",
    "!A"  => "0110001",
    "!M"  => "1110001",
    "-D"  => "0001111",
    "-A"  => "0110011",
    "-M"  => "1110011",
    "D+1" => "0011111",
    "A+1" => "0110111",
    "M+1" => "1110111",
    "D-1" => "0001110",
    "A-1" => "0110010",
    "M-1" => "1110010",
    "D+A" => "0000010",
    "D+M" => "1000010",
    "D-A" => "0010011",
    "D-M" => "1010011",
    "A-D" => "0000111",
    "M-D" => "1000111",
    "D&A" => "0000000",
    "D&M" => "1000000",
    "D|A" => "0010101",
    _     => "1010101", // comp == "D|M"
  }
}

fn dest_to_binary(dest: &str) -> &str {
  match dest {
    ""   => "000",
    "M"  => "001",
    "D"  => "010",
    "DM" => "011",
    "MD" => "011",
    "A"  => "100",
    "AM" => "101",
    "MA" => "101",
    "AD" => "110",
    "DA" => "110",
    _    => "111",  // dest = "ADM"
  }
}