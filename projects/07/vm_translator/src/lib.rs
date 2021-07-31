use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;

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
      Some(_) => {
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

  fs::write(config.output_filename, output)?;

  Ok(())
}

fn generate_output(instructions: &Vec<Instruction>, file_stem: String) -> String {
  let mut output = String::new();
  let mut cnt = 0;

  for instruction in instructions {
    match instruction {
      Instruction::Movement(movement) => {
        match movement {
          Movement::Push(segment, val) => {
            match segment {
              Segment::Argument => output.push_str(&push_argument(val)),
              Segment::Local    => output.push_str(&push_local(val)),
              Segment::Static   => output.push_str(&push_static(val, &file_stem)),
              Segment::Constant => output.push_str(&push_constant(val)),
              Segment::This     => output.push_str(&push_this(val)),
              Segment::That     => output.push_str(&push_that(val)),
              Segment::Pointer  => {
                if val == "0" {
                  output.push_str(push_this_pointer());
                } else {
                  output.push_str(push_that_pointer());
                }
                
              },
              Segment::Temp     => output.push_str(&push_temp(val)),
            }
          },
          Movement::Pop(segment, val) => {
            match segment {
              Segment::Argument => output.push_str(&pop_argument(val)),
              Segment::Local    => output.push_str(&pop_local(val)),
              Segment::Static   => output.push_str(&pop_static(val, &file_stem)),
              Segment::This     => output.push_str(&pop_this(val)),
              Segment::That     => output.push_str(&pop_that(val)),
              Segment::Pointer  => {
                if val == "0" {
                  output.push_str(pop_this_pointer());
                } else {
                  output.push_str(pop_that_pointer());
                }
              },
              Segment::Temp     => output.push_str(&pop_temp(val)),
              _ => continue,
            }
          }
        }
      },
      Instruction::Calculation(calculation) => {
        match calculation {
          Calculation::Arithmetic(arithmetic) => {
            match arithmetic {
              Arithmetic::Add => output.push_str(add()),
              Arithmetic::Sub => output.push_str(sub()),
              Arithmetic::Neg => output.push_str(neg()),
            }
          },
          Calculation::Comparison(comparison) => {
            match comparison {
              Comparison::Eq => output.push_str(&eq(cnt)),
              Comparison::Gt => output.push_str(&gt(cnt)),
              Comparison::Lt => output.push_str(&lt(cnt)),
            }
            cnt += 1;
          },
          Calculation::Logical(logical) => {
            match logical {
              Logical::And => output.push_str(and()),
              Logical::Or  => output.push_str(or()),
              Logical::Not => output.push_str(not()),
            }
          },
        }
      },
    }
  }
  output
}

fn push_argument(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );

  result
}

fn push_local(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@LCL
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );
    
  result
}

fn push_this(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@THIS
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );
    
  result
}

fn push_that(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@THAT
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );
    
  result
}

fn push_this_pointer() -> &'static str {
"\
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
"
}

fn push_that_pointer() -> &'static str {
"\
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
"
}

fn push_temp(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@5
A=A+D
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );
    
  result
}

fn push_constant(val: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", val));
  result.push_str(
"\
D=A
@SP
A=M
M=D
@SP
M=M+1
"
  );

  result
}

fn push_static(val: &str, file_stem: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}.{}\n", val, file_stem));
  result.push_str(
"\
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  );

  result
}

fn pop_argument(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@ARG
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"
  );

  result
}

fn pop_local(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@LCL
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"
  );

  result
}

fn pop_this(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@THIS
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"
  );

  result
}

fn pop_that(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@THAT
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"
  );

  result
}

fn pop_this_pointer() -> &'static str {
"\
@SP
AM=M-1
D=M
@THIS
M=D
"
}

fn pop_that_pointer() -> &'static str {
  "\
  @SP
  AM=M-1
  D=M
  @THAT
  M=D
  "
}

fn pop_temp(offset: &str) -> String {
  let mut result = String::new();
  result.push_str(&format!("@{}\n", offset));
  result.push_str(
"\
D=A
@5
D=D+A
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"
  );

  result
}

fn pop_static(val: &str, file_stem: &str) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@{}.{}
M=D
", val, file_stem)
}

fn add() -> &'static str {
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M+D
@SP
A=M
M=D
@SP
M=M+1
"
}

fn sub() -> &'static str {
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M-D
@SP
A=M
M=D
@SP
M=M+1
"
}

fn neg() -> &'static str {
"\
@SP
AM=M-1
D=-M
@SP
A=M
M=D
@SP
M=M+1
"
}

fn and() -> &'static str {
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M&D
@SP
A=M
M=D
@SP
M=M+1
"
}

fn or() -> &'static str {
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M|D
@SP
 A=M
M=D
@SP
M=M+1
"
}

fn not() -> &'static str {
"\
@SP
AM=M-1
D=!M
@SP
A=M
M=D
@SP
M=M+1
"
}

fn eq(cnt: i32) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M-D
@TRUE{}
D;JEQ
D=0
@SP
A=M
M=D
@SP
M=M+1
@END{}
0;JMP
(TRUE{})
D=-1
@SP
A=M
M=D
@SP
M=M+1
(END{})
", cnt, cnt, cnt, cnt)
}

fn gt(cnt: i32) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M-D
@TRUE{}
D;JGT
D=0
@SP
A=M
M=D
@SP
M=M+1
@END{}
0;JMP
(TRUE{})
D=-1
@SP
A=M
M=D
@SP
M=M+1
(END{})
", cnt, cnt, cnt, cnt)
}

fn lt(cnt: i32) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M-D
@TRUE{}
D;JLT
D=0
@SP
A=M
M=D
@SP
M=M+1
@END{}
0;JMP
(TRUE{})
D=-1
@SP
A=M
M=D
@SP
M=M+1
(END{})
", cnt, cnt, cnt, cnt)
}