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
    BLANK,
    A_INSTRUCTION,
    C_INSTRUCTION,
    LABEL,
    UNKNOWN,
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

fn detect_line_type(line: &str) -> Result<LineType, &'static str> {
    let code = remove_comment(line);
    if code.is_empty() {
        // is comment line
        Ok(LineType::BLANK)
    } else {
        Ok(LineType::UNKNOWN)
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
        // Skip comment lines
        let line_text = line.unwrap();
        let line_type = detect_line_type(&line_text).unwrap();
        println!("{:?}: {}", line_type, line_text);
    }
    Ok(())
}
