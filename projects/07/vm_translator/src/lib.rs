use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use std::fmt;

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

#[derive(Debug)]
enum Instruction {
  Movement(Movement),
  Calculation(Calculation),
}

impl Instruction {
  fn get(s: &str, file_stem: &str) -> Instruction {
    match s.find(" ") {
      Some(_) => {
        let mut parsed = s.split(" ");
        let mv = parsed.next().unwrap();
        let seg = parsed.next().unwrap();
        let val = parsed.next().unwrap();

        Instruction::Movement(Movement::get(mv, seg, val, file_stem))
      },
      None => unsafe { Instruction::Calculation(Calculation::get(s)) }
    }
  }
}

impl fmt::Display for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Instruction::Movement(movement)       => movement.fmt(f),
      Instruction::Calculation(calculation) => calculation.fmt(f),
    }
  }
}

#[derive(Debug)]
enum Movement {
  Push(String),
  Pop(String),
}

impl Movement {
  fn get(mv: &str, seg: &str, val: &str, file_stem: &str) -> Movement {
    match mv {
      "push" => 
        Movement::Push(match seg {
          "argument" => push_argument(val),
          "local"    => push_local(val),
          "static"   => push_static(val, &file_stem),
          "constant" => push_constant(val),
          "this"     => push_this(val),
          "that"     => push_that(val),
          "pointer"  => {
            if val == "0" {
              push_this_pointer()
            } else {
              push_that_pointer()
            }    
              },
          "temp"     => push_temp(val),
          _          => String::new(),
        }),
      _      =>                                     // "pop"
        Movement::Pop(match seg {
          "argument" => pop_argument(val),
          "local"    => pop_local(val),
          "static"   => pop_static(val, &file_stem),
          "this"     => pop_this(val),
          "that"     => pop_that(val),
          "pointer"  => {
            if val == "0" {
              pop_this_pointer()
            } else {
              pop_that_pointer()
            }
          },
          "temp"    => pop_temp(val),
          _         => String::new(),
        }),
    }
  }
}

impl fmt::Display for Movement {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Movement::Push(assembly) => write!(f, "{}", assembly),
      Movement::Pop(assembly)  => write!(f, "{}", assembly),
    }
  }
}

#[derive(Debug)]
enum Calculation {
  Arithmetic(Arithmetic),
  Comparison(Comparison),
  Logical(Logical),
}

impl Calculation {
  unsafe fn get(s: &str) -> Calculation {
    match s {
      "add" | "sub" | "neg" => Calculation::Arithmetic(Arithmetic::get(s)),
      "eq"  | "gt"  | "lt"  => Calculation::Comparison(Comparison::get(s)),
      _                     => Calculation::Logical(Logical::get(s)), // "and" | "or" | "not"
    }
  }
}

impl fmt::Display for Calculation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Calculation::Arithmetic(arithmetic) => arithmetic.fmt(f),
      Calculation::Comparison(comparison) => comparison.fmt(f),
      Calculation::Logical(logical)       => logical.fmt(f),
    }
  }
}

#[derive(Debug)]
enum Arithmetic {
  Add(String),
  Sub(String),
  Neg(String),
}

impl Arithmetic {
  fn get(s: &str) -> Arithmetic {
    match s {
      "add" => Arithmetic::Add(add()),
      "sub" => Arithmetic::Sub(sub()),
      _     => Arithmetic::Neg(neg())  // "neg"
    }
  }
}

impl fmt::Display for Arithmetic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Arithmetic::Add(assembly) => write!(f, "{}", assembly),
      Arithmetic::Sub(assembly)  => write!(f, "{}", assembly),
      Arithmetic::Neg(assembly) => write!(f, "{}", assembly),
    }
  }
}

#[derive(Debug)]
enum Comparison {
  Eq(String),
  Gt(String),
  Lt(String),
}

impl Comparison {
  unsafe fn get(s: &str) -> Comparison {
    static mut COUNT: i32 = 0;

     let comparison = match s {
      "eq" => Comparison::Eq(eq(COUNT)),
      "gt" => Comparison::Gt(gt(COUNT)),
      _    => Comparison::Lt(lt(COUNT)), // "lt"
    };

    COUNT += 1;

    comparison
  }
}

impl fmt::Display for Comparison {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Comparison::Eq(assembly) => write!(f, "{}", assembly),
      Comparison::Gt(assembly)  => write!(f, "{}", assembly),
      Comparison::Lt(assembly) => write!(f, "{}", assembly),
    }
  }
}

#[derive(Debug)]
enum Logical {
  And(String),
  Or(String),
  Not(String),
}

impl Logical {
  fn get(s: &str) -> Logical {
    match s {
      "and" => Logical::And(and()),
      "or"  => Logical::Or(or()),
      _     => Logical::Not(not()) // "not"
    }
  }
}

impl fmt::Display for Logical {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Logical::And(assembly) => write!(f, "{}", assembly),
      Logical::Or(assembly)  => write!(f, "{}", assembly),
      Logical::Not(assembly) => write!(f, "{}", assembly),
    }
  }
}

struct Translator {
  instructions: Vec<Instruction>,
}

impl Translator {
  fn new(input: String, file_stem: &str) -> Translator {
    let mut instructions = Vec::new();
    
    input.split('\n').for_each(|line| {
      let instruction = match line.find("//") {
        Some(comment_index) => line.get(..comment_index).unwrap().trim(),
        None                => line.trim(),
      };

      if !instruction.is_empty() {
        instructions.push(Instruction::get(instruction, file_stem));
      }
    });

    Translator { instructions }
  }

  fn translate(&self) -> String {
    let mut output = String::new();

    self.instructions.iter().for_each(|instruction| {
      output.push_str(&instruction.to_string());
    });

    output
  }
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

fn push_this_pointer() -> String {
  String::from(
"\
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  )
}

fn push_that_pointer() -> String {
  String::from(
"\
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
"
  )
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

fn pop_this_pointer() -> String {
  String::from(
"\
@SP
AM=M-1
D=M
@THIS
M=D
"
  )
}

fn pop_that_pointer() -> String {
  String::from(
"\
@SP
AM=M-1
D=M
@THAT
M=D
"
  )
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

fn add() -> String {
  String::from(
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
  )
}

fn sub() -> String {
  String::from(
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
  )
}

fn neg() -> String {
  String::from(
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
  )
}

fn and() -> String {
  String::from(
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
  )
}

fn or() -> String {
  String::from(
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
  )
}

fn not() -> String {
  String::from(
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
  )
}

unsafe fn eq(cnt: i32) -> String {
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

unsafe fn gt(cnt: i32) -> String {
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

unsafe fn lt(cnt: i32) -> String {
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  let translator = Translator::new(input, &config.file_stem);

  fs::write(config.output_filename, translator.translate())?;

  Ok(())
}