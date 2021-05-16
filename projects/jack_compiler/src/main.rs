use clap::{AppSettings, Clap};
use quick_xml::se::to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;
mod token;

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file_or_dir: String,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let input_path = Path::new(&opts.input_file_or_dir);
    let io_list = jack_compiler::generate_ioset(input_path)?;
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
