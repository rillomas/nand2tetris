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
    let mut reader = BufReader::new(file);
    let mut commands = vec![];
    for line in reader.lines() {
        let line_text = line.unwrap();
        let command = parse_line(&line_text);
        if command.is_some() {
            let cmd = command.unwrap();
            println!(
                "{:?} {:?} {:?} {:?}",
                cmd.command_type(),
                cmd.arithmetic_type(),
                cmd.segment(),
                cmd.index()
            );
            commands.push(cmd);
        }
    }
    Ok(())
}
