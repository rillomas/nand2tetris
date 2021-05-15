use clap::{AppSettings, Clap};
use serde::Serialize;
use serde_xml_rs::to_string;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
mod token;

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file_or_dir: String,
}

struct Reader {
    reader: BufReader<std::fs::File>,
    origin_name: String,
}

#[derive(Serialize)]
struct Tokens {
    tokens: Vec<Box<dyn token::Token>>,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let input_path = Path::new(&opts.input_file_or_dir);
    println!("input: {}", input_path.display());
    let mut output_file_path: PathBuf;
    let mut readers = Vec::new();
    if input_path.is_file() {
        // load single file by single reader
        let file = File::open(input_path)?;
        let reader = Reader {
            reader: BufReader::new(file),
            origin_name: input_path
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
        };
        readers.push(reader);
        output_file_path = PathBuf::from(input_path);
        output_file_path.set_extension("xml");
    } else if input_path.is_dir() {
        // load all files by multiple reader
        for entry in std::fs::read_dir(input_path)? {
            let path = entry.unwrap().path();
            if path.extension().unwrap() == "jack" {
                // only look at vm files
                let origin_name = path
                    .file_stem()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                let file = File::open(path)?;
                let reader = Reader {
                    reader: BufReader::new(file),
                    origin_name: origin_name,
                };
                readers.push(reader);
            }
        }
    } else {
        panic!("Unsupported path specified");
    }

    // apply tokenization and parsing for all jack files
    for reader in readers {
        let mut tokens = Tokens { tokens: Vec::new() };
        let mut context = token::FileContext::new();
        for line in reader.reader.lines() {
            let line_text = line.unwrap();
            let mut tk = token::parse_line(&mut context, &line_text);
            tokens.tokens.append(&mut tk);
        }
        println!("{:?}", tokens.tokens);
        let xml = to_string(&tokens).unwrap();
        println!("{}", xml);
    }

    Ok(())
}
