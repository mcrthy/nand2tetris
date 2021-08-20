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
  Branch(Branch),
}

impl Instruction {
  fn get(s: &str, file_stem: &str) -> Instruction {
    match s.find(" ") {
      Some(_) => {
        let mut parsed = s.split(" ");
        let declaration = parsed.next().unwrap();

        match declaration {
          "push" | "pop" => {
            let seg = parsed.next().unwrap();
            let val = parsed.next().unwrap();

            Instruction::Movement(Movement::get(declaration, seg, val, file_stem))
          },
          _ => {
            let label = parsed.next().unwrap();
            Instruction::Branch(Branch::get(declaration, label))
          }  
        }
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
      Instruction::Branch(branch)           => branch.fmt(f),
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
          "argument" => push_variable("ARG", val),
          "local"    => push_variable("LCL", val),
          "static"   => push_static(val, &file_stem),
          "constant" => push_value(val),
          "this"     => push_variable("THIS", val),
          "that"     => push_variable("THAT", val),
          "pointer"  => {
            if val == "0" {
              push_value("THIS")
            } else {
              push_value("THAT")
            }    
              },
          "temp"     => push_variable("5", val),
          _          => String::new(),
        }),
      _      =>                                     // "pop"
        Movement::Pop(match seg {
          "argument" => pop_variable("ARG", val),
          "local"    => pop_variable("LCL", val),
          "static"   => pop_static(val, &file_stem),
          "this"     => pop_variable("THIS", val),
          "that"     => pop_variable("THAT", val),
          "pointer"  => {
            if val == "0" {
              pop_pointer("THIS")
            } else {
              pop_pointer("THAT")
            }
          },
          "temp"    => pop_variable("5", val),
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
      "add" => Arithmetic::Add(binary_op("ADD")),
      "sub" => Arithmetic::Sub(binary_op("SUB")),
      _     => Arithmetic::Neg(unary_op("NEG"))  // "neg"
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
      "eq" => Comparison::Eq(compare("EQ", COUNT)),
      "gt" => Comparison::Gt(compare("GT", COUNT)),
      _    => Comparison::Lt(compare("LT", COUNT)), // "lt"
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
      "and" => Logical::And(binary_op("AND")),
      "or"  => Logical::Or(binary_op("OR")),
      _     => Logical::Not(unary_op("NOT")) // "not"
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

#[derive(Debug)]
enum Branch {
  Label(String),
  Conditional(String),
  Unconditional(String)
}

impl Branch {
  fn get(s: &str, label: &str) -> Branch {
    let branch = match s {
      "label"   => Branch::Label(format!("({})\n", label)),
      "if-goto" => Branch::Conditional(if_goto(label)),
      _         => Branch::Unconditional(goto(label)),    // val
    };

    branch
  }
}

impl fmt::Display for Branch {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Branch::Label(assembly) => write!(f, "{}", assembly),
      Branch::Conditional(assembly) => write!(f, "{}", assembly),
      Branch::Unconditional(assembly) => write!(f, "{}", assembly),
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

fn push_variable(v_type: &str, offset: &str) -> String {
  let register = match v_type {
    "ARG" | "LCL" | "THIS" | "THAT" => "M",
    _                               => "A",   // temp
  };

  format!(
"\
@{}
D=A
@{}
A={}+D
D=M
@SP
A=M
M=D
@SP
M=M+1
", offset, v_type, register)
}

fn push_value(val: &str) -> String {
  let register = match val {
    "THIS" | "THAT" => "M",
    _               => "A",   // constant
  };

  format!(
"\
@{}
D={}
@SP
A=M
M=D
@SP
M=M+1
", val, register)
}

fn push_static(val: &str, file_stem: &str) -> String {
  format!(
"\
@{}.{}
D=M
@SP
A=M
M=D
@SP
M=M+1
", val, file_stem)
}

fn pop_variable(v_type: &str, offset: &str) -> String {
let register = match v_type {
    "ARG" | "LCL" | "THIS" | "THAT" => "M",
    _                               => "A",   // temp
  };

  format!(
"\
@{}
D=A
@{}
D=D+{}
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
", offset, v_type, register)
}

fn pop_pointer(ptr: &str) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@{}
M=D
", ptr)
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

fn binary_op(op: &str) -> String {
  let op = match op {
    "ADD" => "+",
    "SUB" => "-",
    "AND" => "&",
    _     => "|",   // or
  };

  format!(
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M{}D
@SP
A=M
M=D
@SP
M=M+1
", op)
}

fn unary_op(op: &str) -> String {
  let op = match op {
    "NEG" => "-",
    _     => "!",   // not
  };

  format!(
"\
@SP
AM=M-1
D={}M
@SP
A=M
M=D
@SP
M=M+1
", op)
}

fn compare(comp: &str, cnt: i32) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@SP
AM=M-1
D=M-D
@TRUE.{}
D;J{}
D=0
@SP
A=M
M=D
@SP
M=M+1
@END.{}
0;JMP
(TRUE.{})
D=-1
@SP
A=M
M=D
@SP
M=M+1
(END.{})
", cnt, comp, cnt, cnt, cnt)

}

fn if_goto(label: &str) -> String {
  format!(
"\
@SP
AM=M-1
D=M
@{}
D;JNE
", label)
}

fn goto(label: &str) -> String {
  format!(
"\
@{}
0;JMP
", label)
}

fn f_call(f: &str, n_args: i32, cnt: i32) -> String {
  format!(
"\
// generate a return address
// and push it onto the stack
@RETURN{}
D=A
@SP
A=M
M=D
@SP
M=M+1

// push the LCL of the caller
// onto the stack
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1

// push the ARG of the caller
// onto the stack
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1

// push the THIS of the caller
// onto the stack
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1

// push the THAT of the caller
// onto the stack
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1

// reposition ARG
@SP
D=M
@5
D=D-A
@{}
D=D-A
@LCL
M=D

// reposition LCL
@SP
D=M
@LCL
M=D

// transfer controll to callee
@{}
0;JMP

// inject return address label into the code
({})
", cnt, n_args, f, cnt)
}

fn f_decl(f: &str, n_vars: i32) -> String {
  let mut result = String::new();

  result.push_str(&format!("({})\n", f));

  for _ in 0..n_vars {
    result.push_str(&push_value("0"));
  }

  result
}

fn f_return() -> String {
  String::from(
"\
// get callee's base LCL address
@LCL
D=M

// get the caller's return address
// (five above LCL on the stack)
@5
D=D-A
A=D
D=M
@R13
M=D

// pop the return value into callee's **ARG
// (actually located inside caller function's local stack)
@SP
AM=M-1
D=M
@ARG
A=M
M=D

// set *SP to *ARG+1, just after the return value
@ARG
D=M+1
@SP
M=D

// restore *THAT for the caller
@LCL
D=M
D=D-1
A=D
D=M
@THAT
M=D

// restore *THIS for the caller
@LCL
D=M
D=D-2
A=D
D=M
@THIS
M=D

// restore *ARG for the caller
@LCL
D=M
D=D-3
A=D
D=M
@THIS
M=D

// restore *LCL for the caller
@LCL
D=M
D=D-3
A=D
D=M
@LCL
M=D

// jump to the return address
@R13
A=M
0;JMP
")
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(config.input_filename)?;

  let translator = Translator::new(input, &config.file_stem);

  fs::write(config.output_filename, translator.translate())?;

  Ok(())
}