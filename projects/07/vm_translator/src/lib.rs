use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;

pub struct Config {
  file_stem: String,
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

      let file_stem = String::from(
        Path::new(&input_filename)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap());

      let output_filename = format!("{}.asm", file_stem);

      Ok(Config {
        file_stem,
        input_filename,
        output_filename,
      })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  let instructions = get_instructions(input);

  let output = generate_output(&instructions, config.file_stem);

  Ok(())
}

fn push() -> &'static str {
"\
@SP
A=M
M=D
@SP
M=M+1
"
}

fn pop() -> &'static str {
"\
@SP
AM=M-1
D=M
"
}

fn load_constant(val: &str) -> String {
  format!(
"\
@{}
D=A
", val)
}

fn load_static(val: &str, file_stem: &str) -> String {
  format!(
"\
@{}.{}
D=M
", file_stem, val)
}

fn load_local(val: &str) -> String {
  format!(
"\
@{}
D=A
@LCL
A=M
A=A+D
D=M
", val)
}

fn pop_local(val: &str) -> String {
  format!(
"\
@{}
D=A
@LCL
A=M
D=D+A
@R0
M=D
@SP
AM=M-1
D=M
@R0
A=M
M=D
", val)
}

fn pop_argument(val: &str) -> String {
  format!(
"\
@{}
D=A
@ARG
A=M
D=D+A
@R0
M=D
@SP
AM=M-1
D=M
@R0
A=M
M=D
", val)
}

fn pop_that(val: &str) -> String {
  format!(
"\
@{}
D=A
@THAT
A=M
D=D+A
@R0
M=D
@SP
AM=M-1
D=M
@R0
A=M
M=D
", val)
}

fn pop_this(val: &str) -> String {
  format!(
"\
@{}
D=A
@THIS
A=M
D=D+A
@R0
M=D
@SP
AM=M-1
D=M
@R0
A=M
M=D
", val)
}

fn load_argument(val: &str) -> String {
  format!(
"\
@{}
D=A
@ARG
A=M
A=A+D
D=M
", val)
}

fn load_that(val: &str) -> String {
  format!(
"\
@{}
D=A
@THAT
A=M
A=A+D
D=M
", val)
}


fn load_this(val: &str) -> String {
  format!(
"\
@{}
D=A
@THIS
A=M
A=A+D
D=M
", val)
}

fn load_pointer(val: &str) -> &str {
  if val == "0" {
"\
@THIS
D=M
"
  } else {
"\
@THAT
D=M
"
  }
}

fn load_temp(val: &str) -> String {
  format!(
"\
@5
D=A
@{}
A=A+D
D=M
", val)
}

fn save_static(val: &str, file_stem: &str) -> String {
  format!(
"\
@{}.{}
M=D
", file_stem, val)
}

fn generate_output(instructions: &Vec<Instruction>, file_stem: String) -> String {
  let mut output = String::new();

  for instruction in instructions {
    match instruction {
      Instruction::Movement(movement) => {
        match movement {
          Movement::Push(segment, val) => {
            match segment {
              Segment::Argument => {
                output.push_str(&format!("{}{}", load_argument(val), push()));
              },
              Segment::Local    => {
                output.push_str(&format!("{}{}", load_local(val), push()));
              },
              Segment::Static   => {
                output.push_str(&format!("{}{}", load_static(val, &file_stem), push()));
              },
              Segment::Constant => {
                output.push_str(&format!("{}{}", load_constant(val), push()));
              },
              Segment::This     => {
                output.push_str(&format!("{}{}", load_this(val), push()));
              },
              Segment::That     => {
                output.push_str(&format!("{}{}", load_that(val), push()));
              },
              Segment::Pointer  => {
                output.push_str(&format!("{}{}", load_pointer(val), push()));
              },
              Segment::Temp     => {
                output.push_str(&format!("{}{}", load_temp(val), push()));
              },
            }
          },
          Movement::Pop(segment, val) => {
            match segment {
              Segment::Argument => {
                output.push_str(&pop_argument(val));
              },
              Segment::Local    => {
                output.push_str(&pop_local(val));
              },
              Segment::Static   => {
                output.push_str(&format!("{}{}", pop(), save_static(val, &file_stem)));
              },
              Segment::This     => {
                output.push_str(&pop_this(val));
              },
              Segment::That     => {
                output.push_str(&pop_that(val));
              },
              Segment::Pointer  => {

              },
              Segment::Temp     => {

              },
              _ => continue,
            }
          }
        }
      },
      Instruction::Calculation(calculation) => {
        match calculation {
          Calculation::Arithmetic(arithmetic) => {
            match arithmetic {
              Arithmetic::Add => {

              },
              Arithmetic::Sub => {

              },
              Arithmetic::Neg => {

              },
            }
          },
          Calculation::Comparison(comparison) => {
            match comparison {
              Comparison::Eq => {

              },
              Comparison::Gt => {
  
              },
              Comparison::Lt => {
  
              },
            }
          },
          Calculation::Logical(logical) => {
            match logical {
              Logical::And => {

              },
              Logical::Or  => {

              },
              Logical::Not => {
                
              }
            }
          },
        }
      },
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

enum Segment {
  Argument,
  Local,
  Static,
  Constant,
  This,
  That,
  Pointer,
  Temp,
}

impl Segment {
  fn get(s: &str) -> Segment {
    match s {
      "argument" => Segment::Argument,
      "local"    => Segment::Local,
      "static"   => Segment::Static,
      "constant" => Segment::Constant,
      "this"     => Segment::This,
      "that"     => Segment::That,
      "pointer"  => Segment::Pointer,
      _          => Segment::Temp,    // s == "temp"
    }
  }
}

enum Calculation {
  Arithmetic(Arithmetic),
  Comparison(Comparison),
  Logical(Logical),
}

impl Calculation {
  fn get(s: &str) -> Calculation {
    match s {
      "add" | "sub" | "neg" => Calculation::Arithmetic(Arithmetic::get(s)),
      "eq"  | "gt"  | "lt"  => Calculation::Comparison(Comparison::get(s)),
      _                     => Calculation::Logical(Logical::get(s)), // "and" | "or" | "not"
    }
  }
}

enum Arithmetic {
  Add,
  Sub,
  Neg,
}

impl Arithmetic {
  fn get(s: &str) -> Arithmetic {
    match s {
      "add" => Arithmetic::Add,
      "sub" => Arithmetic::Sub,
      _     => Arithmetic::Neg  // "neg"
    }
  }
}

enum Comparison {
  Eq,
  Gt,
  Lt,
}

impl Comparison {
  fn get(s: &str) -> Comparison {
    match s {
      "eq" => Comparison::Eq,
      "gt" => Comparison::Gt,
      _    => Comparison::Lt, // "lt"
    }
  }
}

enum Logical {
  And,
  Or,
  Not,
}

impl Logical {
  fn get(s: &str) -> Logical {
    match s {
      "and" => Logical::And,
      "or"  => Logical::Or,
      _     => Logical::Not // "not"
    }
  }
}

enum Movement {
  Push(Segment, String),
  Pop(Segment, String),
}

impl Movement {
  fn get(mv: &str, seg: &str, val: &str) -> Movement {
    match mv {
      "push" => Movement::Push(Segment::get(seg), String::from(val)),
      _      => Movement::Pop(Segment::get(seg), String::from(val)),  // "pop"
    }
  }
}

enum Instruction {
  Movement(Movement),
  Calculation(Calculation),
}

impl Instruction {
  fn get(s: &str) -> Instruction {
    match s.find(" ") {
      Some(space_index) => {
        let mut parsed = s.split(" ");
        let mv = parsed.next().unwrap();
        let seg = parsed.next().unwrap();
        let val = parsed.next().unwrap();

        Instruction::Movement(Movement::get(mv, seg, val))
      },
      None => Instruction::Calculation(Calculation::get(s))
    }
  }
}