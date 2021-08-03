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

struct Assembly {
  instructions: Vec<Instruction>,
}

impl Assembly {
  fn new(input: String) -> Assembly {
    let mut instructions = Vec::new();

    input.split('\n').for_each(|line| {
      let instruction = match line.find("//") {
        Some(comment_index) => line.get(..comment_index).unwrap().trim(),
        None                => line.trim(),
      };

      if !instruction.is_empty() {
        instructions.push(Instruction::get(instruction));
      }
    });

    Assembly { instructions }
  }
}

struct Assembler {
  symbol_table: HashMap<String, i32>,
}

impl Assembler {
  fn new() -> Assembler {
    let mut symbol_table: HashMap<String, i32> = HashMap::new();

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

    Assembler { symbol_table }
  }

  fn translate(&self, instructions: &Vec<Instruction>) -> String {
    let mut symbol_table = self.symbol_table.clone();

    // add labels to symbol table
    let mut line_number = 0;
    instructions.iter().for_each(|instruction| {
      match instruction {
        Instruction::LInstruction(label) => {
          symbol_table.insert(
            label.clone(),
            line_number
          );

        },
        _ => line_number += 1,
      }
    });

    // translate instructions
    let mut output = String::new();
    let mut curr_ram_loc = 16;
    instructions.iter().for_each(|instruction| {
      let mut translation = String::new();

      if let Instruction::AInstruction(a_instruction) = instruction {
        match a_instruction {
          AInstruction::Num(binary) => translation = format!("{}\n", binary),
          AInstruction::Var(name)   => {                                
            if let Some(value) = symbol_table.get(name) {
              translation = format!("{:016b}\n", value);
            } else {
              symbol_table.insert(
                name.clone(),
                curr_ram_loc,
              );

              translation = format!("{:016b}\n", curr_ram_loc);

              curr_ram_loc += 1;
            }
          },
        }
      } else if let Instruction::CInstruction(binary) = instruction {
        translation = format!("{}\n", binary);
      }

      output.push_str(&translation);
    });

    output
  }
}

#[derive(Clone, PartialEq)]
enum Instruction {
  AInstruction(AInstruction),
  LInstruction(String),
  CInstruction(String),
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

#[derive(Clone, PartialEq)]
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
    ""          => "000",
    "M"         => "001",
    "D"         => "010",
    "DM" | "MD" => "011",
    "A"         => "100",
    "AM" | "MA" => "101",
    "AD" | "DA" => "110",
    _    => "111",  // dest = "ADM"
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  let assembly = Assembly::new(input);
  let assembler = Assembler::new();

  fs::write(config.output_filename, assembler.translate(&assembly.instructions))?;
  Ok(())
}