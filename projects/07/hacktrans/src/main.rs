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
type CommandID = u32;

/// Type of arithmetic command
#[derive(Debug, Copy, Clone)]
enum ArithmeticType {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
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

const NULL_ID: CommandID = 0;
const COMMENT_SYMBOL: &str = "//";

// const STATIC_VAR_START: u32 = 16;
// const STATIC_VAR_SIZE: u32 = 240;
// const STACK_START_OFFSET: u32 = 256;
// const STACK_SIZE: u32 = 1792;
// const HEAP_START_OFFSET: u32 = 2048;
// const HEAP_SIZE: u32 = 14436;

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

/// Counter for specific commands.
/// We need to count the number to create a unique ID to use as jump labels in each command.
/// Without this we will have clashing jump lables each time we use eq, gt, and lt.
struct CommandCounter {
    eq: CommandID,
    gt: CommandID,
    lt: CommandID,
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
    /// Unique ID of command within the same command group.
    /// This is used to create unique jump labels per command.
    /// If this is 0 (NULL_ID) it means it is not used for this command
    id: CommandID,
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
            ArithmeticType::Eq => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsEq.{0}
D;JEQ
D=-1
(IsEq.{0})
@SP
A=M-1
A=A-1
M=!D
D=A+1
@SP
M=D
",
                self.id
            )),
            ArithmeticType::Lt => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsGe.{0}
D;JGE
D=-1
@WriteLtOutput.{0}
0;JMP
(IsGe.{0})
D=0
(WriteLtOutput.{0})
@SP
A=M-1
A=A-1
M=D
D=A+1
@SP
M=D
",
                self.id
            )),
            ArithmeticType::Gt => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsGt.{0}
D;JGT
D=0
@WriteGtOutput.{0}
0;JMP
(IsGt.{0})
D=-1
(WriteGtOutput.{0})
@SP
A=M-1
A=A-1
M=D
D=A+1
@SP
M=D
",
                self.id
            )),
            _ => Err("Unsupported Arithmetic type"),
        }
    }
}

fn parse_line(line: &str, counter: &mut CommandCounter) -> Option<Box<dyn Command>> {
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
            id: NULL_ID,
        })),
        "sub" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Sub,
            id: NULL_ID,
        })),
        "neg" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Neg,
            id: NULL_ID,
        })),
        "eq" => {
            counter.eq += 1; // We increment first because 0 is reserved for null
            Some(Box::new(ArithmeticCommand {
                command: CommandType::Arithmetic,
                arithmetic: ArithmeticType::Eq,
                id: counter.eq,
            }))
        }
        "gt" => {
            counter.gt += 1; // We increment first because 0 is reserved for null
            Some(Box::new(ArithmeticCommand {
                command: CommandType::Arithmetic,
                arithmetic: ArithmeticType::Gt,
                id: counter.gt,
            }))
        }
        "lt" => {
            counter.lt += 1; // We increment first because 0 is reserved for null
            Some(Box::new(ArithmeticCommand {
                command: CommandType::Arithmetic,
                arithmetic: ArithmeticType::Lt,
                id: counter.lt,
            }))
        }
        "and" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::And,
            id: NULL_ID,
        })),
        "or" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Or,
            id: NULL_ID,
        })),
        "not" => Some(Box::new(ArithmeticCommand {
            command: CommandType::Arithmetic,
            arithmetic: ArithmeticType::Not,
            id: NULL_ID,
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
    let mut counter = CommandCounter {
        eq: 0,
        lt: 0,
        gt: 0,
    };
    for line in reader.lines() {
        let line_text = line.unwrap();
        let command = parse_line(&line_text, &mut counter);
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
