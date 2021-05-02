use clap::{AppSettings, Clap};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
mod command;
use command::Arithmetic;
use command::ArithmeticType;
use command::Command;
use command::CommandType;
use command::MemoryAccess;
use command::ProgramFlow;
use command::NULL_ID;

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file: String,
}
const COMMENT_SYMBOL: &str = "//";
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

fn parse_line(line: &str, counter: &mut command::Counter) -> Option<Box<dyn Command>> {
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
        "push" => Some(Box::new(command::MemoryAccess::new(
            CommandType::Push,
            itr.next().unwrap(),
            itr.next().unwrap(),
        ))),
        "pop" => Some(Box::new(MemoryAccess::new(
            CommandType::Pop,
            itr.next().unwrap(),
            itr.next().unwrap(),
        ))),
        "add" => Some(Box::new(Arithmetic::new(ArithmeticType::Add, NULL_ID))),
        "sub" => Some(Box::new(Arithmetic::new(ArithmeticType::Sub, NULL_ID))),
        "neg" => Some(Box::new(Arithmetic::new(ArithmeticType::Neg, NULL_ID))),
        "eq" => {
            counter.eq += 1; // We increment first because 0 is reserved for null
            Some(Box::new(Arithmetic::new(ArithmeticType::Eq, counter.eq)))
        }
        "gt" => {
            counter.gt += 1; // We increment first because 0 is reserved for null
            Some(Box::new(Arithmetic::new(ArithmeticType::Gt, counter.gt)))
        }
        "lt" => {
            counter.lt += 1; // We increment first because 0 is reserved for null
            Some(Box::new(Arithmetic::new(ArithmeticType::Lt, counter.lt)))
        }
        "and" => Some(Box::new(Arithmetic::new(ArithmeticType::And, NULL_ID))),
        "or" => Some(Box::new(Arithmetic::new(ArithmeticType::Or, NULL_ID))),
        "not" => Some(Box::new(Arithmetic::new(ArithmeticType::Not, NULL_ID))),
        "label" => Some(Box::new(ProgramFlow::new(
            CommandType::Label,
            itr.next().unwrap().to_string(),
        ))),
        "goto" => Some(Box::new(ProgramFlow::new(
            CommandType::GoTo,
            itr.next().unwrap().to_string(),
        ))),
        "if-goto" => Some(Box::new(ProgramFlow::new(
            CommandType::If,
            itr.next().unwrap().to_string(),
        ))),
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
    let mut counter = command::Counter {
        eq: 0,
        lt: 0,
        gt: 0,
    };
    for line in reader.lines() {
        let line_text = line.unwrap();
        let command = parse_line(&line_text, &mut counter);
        if command.is_some() {
            let cmd = command.unwrap();
            // println!("{:?}", cmd);
            commands.push(cmd);
        }
    }

    // convert VM commands to hack asm
    let mut out_file = File::create(output_file_path).unwrap();
    let prefix = input_file_path
        .file_stem()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();
    for cmd in commands {
        let _written = out_file
            .write(cmd.to_asm_text(&prefix).unwrap().as_bytes())
            .unwrap();
    }
    // Add loop at the end to avoid code injection
    let _written = out_file.write(LOOP_STR.as_bytes());
    Ok(())
}
