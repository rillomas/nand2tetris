use clap::{AppSettings, Clap};
use quick_xml::se::to_string;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
mod token;

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file_or_dir: String,
}

struct IOSet {
    input: BufReader<std::fs::File>,
    input_file: PathBuf,
    output_file: PathBuf,
}

/// Read a file path or directory of files to get valid input/output file paths
fn generate_ioset(input_file_or_dir: &str) -> Result<Vec<IOSet>, std::io::Error> {
    let input_path = Path::new(input_file_or_dir);
    let mut file_list = Vec::new();
    if input_path.is_file() {
        // load single file by single reader
        let file = File::open(input_path)?;
        let origin_name = input_path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let mut output_file_path = PathBuf::from(input_path);
        let out_name = format!("My{}.xml", origin_name);
        output_file_path.set_file_name(out_name);
        let set = IOSet {
            input: BufReader::new(file),
            input_file: input_path.to_owned(),
            output_file: output_file_path,
        };
        file_list.push(set);
        Ok(file_list)
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
                let mut output_file_path = path.clone();
                let file = File::open(&path)?;
                let out_name = format!("My{}.xml", origin_name);
                output_file_path.set_file_name(out_name);
                let set = IOSet {
                    input: BufReader::new(file),
                    input_file: path.to_owned(),
                    output_file: output_file_path,
                };
                file_list.push(set);
            }
        }
        Ok(file_list)
    } else {
        panic!("Unsupported path specified");
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let io_list = generate_ioset(&opts.input_file_or_dir)?;
    // apply tokenization and parsing for all jack files
    for mut io in io_list {
        println!("input: {}", &io.input_file.display());
        println!("output: {}", &io.output_file.display());
        let tokens = token::generate_token_list(&mut io.input);
        let xml = to_string(&tokens).unwrap();
        // let mut out_file = File::create(io.output_file)?;
        // out_file.write(xml.as_bytes())?;
        println!("{}", xml);
    }

    Ok(())
}
