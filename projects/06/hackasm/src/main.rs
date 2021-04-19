use clap::{AppSettings, Clap};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    jmp: Option<String>,
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
                    jmp: None,
                }
            } else {
                // no dest, has jmp
                let comp_jmp: Vec<_> = line.split(jmp_delimiter).collect();
                CInstruction {
                    comp: comp_jmp[0].to_string(),
                    dest: None,
                    jmp: Some(comp_jmp[1].to_string()),
                }
            }
        } else {
            if jmp_position == None {
                // has dest, no jmp
                let dest_comp: Vec<_> = line.split(dest_delimiter).collect();
                CInstruction {
                    comp: dest_comp[0].to_string(),
                    dest: Some(dest_comp[1].to_string()),
                    jmp: None,
                }
            } else {
                // has both dest and jmp
                let dest_comp_jmp: Vec<_> = line.split(dest_delimiter).collect();
                let comp_jmp: Vec<_> = dest_comp_jmp[1].split(jmp_delimiter).collect();
                CInstruction {
                    comp: comp_jmp[0].to_string(),
                    dest: Some(dest_comp_jmp[0].to_string()),
                    jmp: Some(comp_jmp[1].to_string()),
                }
            }
        }
    }
}

fn remove_comment(line: &str) -> &str {
    match line.find("//") {
        Some(pos) => {
            // create substr based on comment position
            let (first, _last) = line.split_at(pos);
            first
        }
        // No comment so we just use the original line
        None => line,
    }
}

fn parse_line(line: &str) -> Result<LineType, &'static str> {
    let trimmed = line.trim();
    let code = remove_comment(trimmed);
    if code.is_empty() {
        // is comment line
        return Ok(LineType::Blank);
    }
    let first_char = code.chars().nth(0);
    match first_char {
        Some('@') => Ok(LineType::AInstruction),
        Some('(') => Ok(LineType::Label),
        _ => {
            let cinst = CInstruction::new(code);
            println!("{:?}", cinst);
            Ok(LineType::CInstruction)
        }
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let input_file_path = Path::new(&opts.input_file);
    let mut output_file_path = PathBuf::from(input_file_path);
    output_file_path.set_extension("hack");
    println!("input: {}", input_file_path.display());
    println!("output: {}", output_file_path.display());
    let file = File::open(input_file_path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line_text = line.unwrap();
        let line_type = parse_line(&line_text).unwrap();
        println!("{:?}: {}", line_type, line_text);
    }
    Ok(())
}
