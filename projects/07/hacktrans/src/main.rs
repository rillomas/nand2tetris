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
    fn to_asm_text(&self) -> Result<String, String>;
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
    fn to_asm_text(&self) -> Result<String, String> {
        match self.command {
            CommandType::Push => match self.segment {
                SegmentType::Constant => {
                    // push index value to global stack
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
                SegmentType::Local => {
                    // push value from local segment to global stack
                    let str = format!(
                        "@{}
D=A
@LCL
A=D+M
D=M
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
                SegmentType::Argument => {
                    // push value from argument segment to global stack
                    let str = format!(
                        "@{}
D=A
@ARG
A=D+M
D=M
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
                SegmentType::This => {
                    // push value from this segment to global stack
                    let str = format!(
                        "@{}
D=A
@THIS
A=D+M
D=M
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
                SegmentType::That => {
                    // push value from that segment to global stack
                    let str = format!(
                        "@{}
D=A
@THAT
A=D+M
D=M
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
                SegmentType::Temp => {
                    // push value from temp segment to global stack
                    let str = format!(
                        "@{}
D=A
@R5
A=D+A
D=M
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
                SegmentType::Pointer => {
                    // push value from pointer segment to global stack
                    let str = format!(
                        "@{}
D=A
@R3
A=D+A
D=M
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
                // SegmentType::Static => {}
                _other => Err(format!("Unsupported memory segment for Push: {:?}", _other)),
            },
            CommandType::Pop => match self.segment {
                SegmentType::Local => {
                    // move value from global stack to local segment
                    let str = format!(
                        "@{}
D=A
@LCL
D=D+M
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                SegmentType::Argument => {
                    // move value from global stack to argument segment
                    let str = format!(
                        "@{}
D=A
@ARG
D=D+M
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                SegmentType::This => {
                    // move value from global stack to this segment
                    let str = format!(
                        "@{}
D=A
@THIS
D=D+M
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                SegmentType::That => {
                    // move value from global stack to that segment
                    let str = format!(
                        "@{}
D=A
@THAT
D=D+M
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                SegmentType::Temp => {
                    // move value from global stack to temp segment (R5 to R12)
                    let str = format!(
                        "@{}
D=A
@R5
D=D+A
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                SegmentType::Pointer => {
                    // move value from global stack to pointer segment (R3 to R4)
                    let str = format!(
                        "@{}
D=A
@R3
D=D+A
@targetAddr
M=D
@SP
AM=M-1
D=M
@targetAddr
A=M
M=D
",
                        self.index
                    );
                    Ok(str.to_string())
                }
                // SegmentType::Static => {}
                _other => Err(format!("Unsupported memory segment for Pop: {:?}", _other)),
            },
            _other => Err(format!("Unsupported MemoryAccessCommand: {:?}", _other)),
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
            "temp" => SegmentType::Temp,
            "pointer" => SegmentType::Pointer,
            _other => panic!("Unknown segment specified: {:?}", _other),
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
    fn to_asm_text(&self) -> Result<String, String> {
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
            CommandType::Pop,
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
