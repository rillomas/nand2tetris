use clap::{AppSettings, Clap};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, Write};
use std::path::{Path, PathBuf};

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file: String,
}

type MemoryIndex = u32;

/// Type of arithmetic command
#[derive(Debug, Copy, Clone)]
enum ArithmeticType {
    Add,
    Sub,
    Neg,
    /// Negate
    Eq,
    /// Equal
    Gt,
    /// Greater
    Lt,
    /// Little
    And,
    Or,
    Not,
}

/// Type of VM command
#[derive(Debug, Copy, Clone)]
enum CommandType {
    Arithmetic,
    Push,
    Pop,
    Label,
    GoTo,
    If,
    Function,
    Return,
    Call,
}

/// Type of segment for VM memory access (push, pop)
#[derive(Debug, Copy, Clone)]
enum SegmentType {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

const COMMENT_SYMBOL: &str = "//";

const STATIC_VAR_START: u32 = 16;
const STATIC_VAR_SIZE: u32 = 240;
const STACK_START_OFFSET: u32 = 256;
const STACK_SIZE: u32 = 1792;
const HEAP_START_OFFSET: u32 = 2048;
const HEAP_SIZE: u32 = 14436;

const ADD_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D+M
D=A+1
@SP
M=D
";

const SUB_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=M-D
D=A+1
@SP
M=D
";

const AND_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D&M
D=A+1
@SP
M=D
";

const OR_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D|M
D=A+1
@SP
M=D
";

const NEG_STR: &'static str = "@SP
A=M
A=A-1
D=M
M=-M
D=A+1
@SP
M=D
";

const NOT_STR: &'static str = "@SP
A=M
A=A-1
D=M
M=!M
D=A+1
@SP
M=D
";

/// EQ is !Xor(x,y)
const EQ_STR: &'static str = "@SP
A=M
A=A-1
D=M
@y
M=D
@noty
M=!D
@SP
A=M-1
D=M
@x
M=D
@notx
M=!D
@y
D=M
@notx
D=D&M
@andYNotX
M=D
@noty
D=M
@x
D=D&M
@andNotYX
M=D
@andYNotX
D=M
@andNotYX
D=D|M
D=!D
@SP
A=M-1
A=A-1
M=D
D=A+1
@SP
M=D
";

const LOOP_STR: &'static str = "(LOOP_AT_END)
@LOOP_AT_END
0;JMP
";

fn remove_comment(line: &str) -> &str {
    match line.find(COMMENT_SYMBOL) {
        Some(pos) => {
            // create substr based on comment position
            let (first, _last) = line.split_at(pos);
            first
        }
        // No comment so we just use the original line
        None => line,
    }
}

trait Command {
    /// Returns current command's command type
    fn command_type(&self) -> CommandType;
    /// Returns current command's segment for push/pop command. Other commands will return None
    fn segment(&self) -> Option<SegmentType>;
    /// Returns current command's arithmetic type for arithmetic command. Other commands will return None
    fn arithmetic_type(&self) -> Option<ArithmeticType>;
    /// Returns target memory index for push/pop command. Other commands will return none
    fn index(&self) -> Option<MemoryIndex>;
    /// Convert command to corresponding hask asm text
    fn to_asm_text(&self) -> Result<String, &'static str>;
}

struct MemoryAccessCommand {
    command: CommandType,
    segment: SegmentType,
    index: MemoryIndex,
}

impl Command for MemoryAccessCommand {
    fn command_type(&self) -> CommandType {
        self.command
    }
    fn segment(&self) -> Option<SegmentType> {
        Some(self.segment)
    }
    fn arithmetic_type(&self) -> Option<ArithmeticType> {
        None
    }
    fn index(&self) -> Option<MemoryIndex> {
        Some(self.index)
    }
    fn to_asm_text(&self) -> Result<String, &'static str> {
        match self.command {
            CommandType::Push => match self.segment {
                SegmentType::Constant => {
                    let str = format!(
                        "@{}
D=A
@SP
A=M
M=D
@SP
M=M+1
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                _ => Err("Unsupported memory segment"),
            },
            _ => Err("Unsupported MemoryAccessCommand"),
        }
    }
}

impl MemoryAccessCommand {
    fn new(command: CommandType, segment: &str, index: &str) -> MemoryAccessCommand {
        let seg = match segment {
            "argument" => SegmentType::Argument,
            "local" => SegmentType::Local,
            "static" => SegmentType::Static,
            "constant" => SegmentType::Constant,
            "this" => SegmentType::This,
            "that" => SegmentType::That,
            "pointer" => SegmentType::Pointer,
            _ => panic!("Unknown segment specified"),
        };
        let idx = str::parse::<MemoryIndex>(index);
        MemoryAccessCommand {
            command: command,
            segment: seg,
            index: idx.unwrap(),
        }
    }
}

struct ArithmeticCommand {
    command: CommandType,
    arithmetic: ArithmeticType,
}

impl Command for ArithmeticCommand {
    fn command_type(&self) -> CommandType {
        self.command
    }
    fn segment(&self) -> Option<SegmentType> {
        None
    }
    fn arithmetic_type(&self) -> Option<ArithmeticType> {
        Some(self.arithmetic)
    }
    fn index(&self) -> Option<MemoryIndex> {
        None
    }
    fn to_asm_text(&self) -> Result<String, &'static str> {
        match self.arithmetic {
            ArithmeticType::Add => Ok(ADD_STR.to_string()),
            ArithmeticType::Sub => Ok(SUB_STR.to_string()),
            ArithmeticType::And => Ok(AND_STR.to_string()),
            ArithmeticType::Or => Ok(OR_STR.to_string()),
            ArithmeticType::Neg => Ok(NEG_STR.to_string()),
            ArithmeticType::Not => Ok(NOT_STR.to_string()),
            ArithmeticType::Eq => Ok(EQ_STR.to_string()),
            // _ => Err("Unsupported Arithmetic type"),
            _ => Ok("".to_string()),
        }
    }
}

fn parse_line(line: &str) -> Option<Box<dyn Command>> {
    let mut code = remove_comment(line);
    code = code.trim();
    if code.is_empty() {
        // is comment line
        return None;
    }
    let mut itr = code.split_whitespace();
    // We should always have a valid first clause
    let command = itr.next().unwrap();
    match command {
        "push" => Some(Box::new(MemoryAccessCommand::new(
            CommandType::Push,
            itr.next().unwrap(),
            itr.next().unwrap(),
        ))),
        "pop" => Some(Box::new(MemoryAccessCommand::new(
            CommandType::Push,
            itr.next().unwrap(),
            itr.next().unwrap(),
        ))),
        "add" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Add,
        })),
        "sub" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Sub,
        })),
        "neg" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Neg,
        })),
        "eq" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Eq,
        })),
        "gt" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Gt,
        })),
        "lt" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Lt,
        })),
        "and" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::And,
        })),
        "or" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Or,
        })),
        "not" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Not,
        })),
        _ => None,
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let input_file_path = Path::new(&opts.input_file);
    let mut output_file_path = PathBuf::from(input_file_path);
    output_file_path.set_extension("asm");
    println!("input: {}", input_file_path.display());
    println!("output: {}", output_file_path.display());
    let file = File::open(input_file_path)?;
    let reader = BufReader::new(file);
    let mut commands = vec![];
    for line in reader.lines() {
        let line_text = line.unwrap();
        let command = parse_line(&line_text);
        if command.is_some() {
            let cmd = command.unwrap();
            // println!(
            //     "{:?} {:?} {:?} {:?}",
            //     cmd.command_type(),
            //     cmd.arithmetic_type(),
            //     cmd.segment(),
            //     cmd.index()
            // );
            commands.push(cmd);
        }
    }

    // convert VM commands to hack asm
    let mut out_file = File::create(output_file_path)?;
    for cmd in commands {
        let _written = out_file
            .write(cmd.to_asm_text().unwrap().as_bytes())
            .unwrap();
    }
    // Add loop at the end to avoid code injection
    let _written = out_file.write(LOOP_STR.as_bytes());
    Ok(())
}
