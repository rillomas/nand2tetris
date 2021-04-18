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
    COMMENT,
    A_INSTRUCTION,
    C_INSTRUCTION,
    LABEL,
    UNKNOWN,
}

fn detect_line_type(line: &str) -> Result<LineType, &'static str> {
    Ok(LineType::UNKNOWN)
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
