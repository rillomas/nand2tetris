use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "Masato Nakasaka <rillomas@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short)]
    input_file: String,
    #[clap(short)]
    output_file: String,
}

fn main() {
    let opts = Opts::parse();
    println!("input: {} output: {}", opts.input_file, opts.output_file);
}
