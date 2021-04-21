use clap::{AppSettings, Clap};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file: String,
}

/// Type of line from asm code
#[derive(Debug)]
enum LineType {
    Blank,
    AInstruction,
    CInstruction,
    Label,
}

#[derive(Debug)]
struct CInstruction {
    comp: String,
    dest: Option<String>,
    jump: Option<String>,
}

#[derive(Debug)]
struct AInstruction {
    value: u16,
}

const A_INSTRUCTION_SYMBOL: char = '@';
const COMMENT_SYMBOL: &str = "//";
const PREDEFINED_SYMBOL: [(&str, u16); 23] = [
    ("SP", 0),
    ("LCL", 1),
    ("ARG", 2),
    ("THIS", 3),
    ("THAT", 4),
    ("R0", 0),
    ("R1", 1),
    ("R2", 2),
    ("R3", 3),
    ("R4", 4),
    ("R5", 5),
    ("R6", 6),
    ("R7", 7),
    ("R7", 8),
    ("R9", 9),
    ("R10", 10),
    ("R11", 11),
    ("R12", 12),
    ("R13", 13),
    ("R14", 14),
    ("R15", 15),
    ("SCREEN", 0x4000),
    ("KBD", 0x6000),
];

trait Instruction {
    /// Convert instruction to binary text (hack format)
    fn to_binary_text(&self) -> Result<String, &'static str>;
}

impl Instruction for CInstruction {
    fn to_binary_text(&self) -> Result<String, &'static str> {
        let mut output = String::from("111");
        match self.comp.as_str() {
            "0" => output.push_str("0101010"),
            "1" => output.push_str("0111111"),
            "-1" => output.push_str("0111010"),
            "D" => output.push_str("0001100"),
            "A" => output.push_str("0110000"),
            "M" => output.push_str("1110000"),
            "!D" => output.push_str("0001101"),
            "!A" => output.push_str("0110001"),
            "!M" => output.push_str("1110001"),
            "-D" => output.push_str("0001111"),
            "-A" => output.push_str("0110011"),
            "-M" => output.push_str("1110011"),
            "D+1" => output.push_str("0011111"),
            "A+1" => output.push_str("0110111"),
            "M+1" => output.push_str("1110111"),
            "D-1" => output.push_str("0001110"),
            "A-1" => output.push_str("0110010"),
            "M-1" => output.push_str("1110010"),
            "D+A" => output.push_str("0000010"),
            "D+M" => output.push_str("1000010"),
            "D-A" => output.push_str("0010011"),
            "D-M" => output.push_str("1010011"),
            "A-D" => output.push_str("0000111"),
            "M-D" => output.push_str("1000111"),
            "D&A" => output.push_str("0000000"),
            "D&M" => output.push_str("1000000"),
            "D|A" => output.push_str("0010101"),
            "D|M" => output.push_str("1010101"),
            _ => return Err("Unknown comp"),
        }
        match self.dest.as_deref() {
            None => output.push_str("000"),
            Some("M") => output.push_str("001"),
            Some("D") => output.push_str("010"),
            Some("MD") => output.push_str("011"),
            Some("A") => output.push_str("100"),
            Some("AM") => output.push_str("101"),
            Some("AD") => output.push_str("110"),
            Some("AMD") => output.push_str("111"),
            _ => return Err("Unknown dest"),
        }
        match self.jump.as_deref() {
            None => output.push_str("000\n"),
            Some("JGT") => output.push_str("001\n"),
            Some("JEQ") => output.push_str("010\n"),
            Some("JGE") => output.push_str("011\n"),
            Some("JLT") => output.push_str("100\n"),
            Some("JNE") => output.push_str("101\n"),
            Some("JLE") => output.push_str("110\n"),
            Some("JMP") => output.push_str("111\n"),
            _ => return Err("Unknown jump"),
        }
        Ok(output)
    }
}

impl CInstruction {
    fn new(line: &str) -> CInstruction {
        let dest_delimiter = '=';
        let jmp_delimiter = ';';
        let dest_position = line.find(dest_delimiter);
        let jmp_position = line.find(jmp_delimiter);
        if dest_position == None {
            if jmp_position == None {
                // no dest, no jmp
                CInstruction {
                    comp: line.to_string(),
                    dest: None,
                    jump: None,
                }
            } else {
                // no dest, has jmp
                let comp_jmp: Vec<_> = line.split(jmp_delimiter).collect();
                CInstruction {
                    comp: comp_jmp[0].to_string(),
                    dest: None,
                    jump: Some(comp_jmp[1].to_string()),
                }
            }
        } else {
            if jmp_position == None {
                // has dest, no jmp
                let dest_comp: Vec<_> = line.split(dest_delimiter).collect();
                CInstruction {
                    comp: dest_comp[1].to_string(),
                    dest: Some(dest_comp[0].to_string()),
                    jump: None,
                }
            } else {
                // has both dest and jmp
                let dest_comp_jmp: Vec<_> = line.split(dest_delimiter).collect();
                let comp_jmp: Vec<_> = dest_comp_jmp[1].split(jmp_delimiter).collect();
                CInstruction {
                    comp: comp_jmp[0].to_string(),
                    dest: Some(dest_comp_jmp[0].to_string()),
                    jump: Some(comp_jmp[1].to_string()),
                }
            }
        }
    }
}

impl Instruction for AInstruction {
    fn to_binary_text(&self) -> Result<String, &'static str> {
        Ok(format!("{:016b}\n", self.value))
    }
}

impl AInstruction {
    fn new(line: &str, symbol_table: &HashMap<&str, u16>) -> AInstruction {
        let splitten: Vec<_> = line.split(A_INSTRUCTION_SYMBOL).collect();
        let address_or_symbol = splitten[1];
        let maybe_address = str::parse::<u16>(address_or_symbol);
        if maybe_address.is_ok() {
            // A instruction is direct address
            let value = maybe_address.unwrap();
            AInstruction { value: value }
        } else {
            // A instruction is a symbol
            // Lookup table to get address
            let address = symbol_table.get(address_or_symbol).unwrap();
            AInstruction { value: *address }
        }
    }
}

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

fn parse_line(
    line: &str,
    symbol_table: &HashMap<&str, u16>,
    instruction_output: &mut Vec<Box<dyn Instruction>>,
) -> Result<LineType, &'static str> {
    let trimmed = line.trim();
    let code = remove_comment(trimmed);
    if code.is_empty() {
        // is comment line
        return Ok(LineType::Blank);
    }
    let first_char = code.chars().nth(0);
    match first_char {
        Some(A_INSTRUCTION_SYMBOL) => {
            let ainst = AInstruction::new(code, symbol_table);
            // println!("{:?}", ainst);
            instruction_output.push(Box::new(ainst));
            Ok(LineType::AInstruction)
        }
        Some('(') => Ok(LineType::Label),
        _ => {
            let cinst = CInstruction::new(code);
            // println!("{:?}", cinst);
            instruction_output.push(Box::new(cinst));
            Ok(LineType::CInstruction)
        }
    }
}

fn init_symbol_table(table: &mut HashMap<&str, u16>, reader: &BufReader<std::fs::File>) {}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let input_file_path = Path::new(&opts.input_file);
    let mut output_file_path = PathBuf::from(input_file_path);
    output_file_path.set_extension("hack");
    println!("input: {}", input_file_path.display());
    println!("output: {}", output_file_path.display());
    let file = File::open(input_file_path)?;
    let reader = BufReader::new(file);
    let mut instructions = vec![];
    let mut symbol_table: HashMap<&str, u16> = PREDEFINED_SYMBOL.iter().cloned().collect();
    init_symbol_table(&mut symbol_table, &reader);
    for line in reader.lines() {
        let line_text = line.unwrap();
        let _line_type = parse_line(&line_text, &symbol_table, &mut instructions).unwrap();
        println!("{:?}: {}", _line_type, line_text);
    }
    // let mut out_file = File::create(output_file_path)?;
    // for inst in instructions {
    //     let written = out_file
    //         .write(inst.to_binary_text().unwrap().as_bytes())
    //         .unwrap();
    //     assert_eq!(written, 17); // 16 chars + new line
    // }
    Ok(())
}
