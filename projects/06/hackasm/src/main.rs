use clap::{AppSettings, Clap};
use std::path::{Path, PathBuf};

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file: String,
}

fn main() {
    let opts = Opts::parse();
    let input_file = Path::new(&opts.input_file);
    let mut output_file = PathBuf::from(input_file);
    output_file.set_extension("hack");
    println!("input: {}", input_file.display());
    println!("output: {}", output_file.display());
}
