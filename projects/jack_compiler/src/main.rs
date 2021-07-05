use clap::{AppSettings, Clap};
use std::fs::File;
use std::io::Write;
use std::path::Path;

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
    // compile all files
    for mut io in io_list {
        println!("input: {}", &io.input_file.display());
        let mut output_file_path = io.input_file.clone();
        let origin_name = jack_compiler::get_origin_name(&io.input_file).unwrap();
        let out_name = format!("{}.vm", origin_name);
        output_file_path.set_file_name(out_name);
        println!("output: {}", &output_file_path.display());
        let mut ctx = jack_compiler::parser::ParseInfo::new();
        let class = jack_compiler::parser::parse_file(&mut ctx, &mut io.input).unwrap();
        let vm = class.compile(&ctx).unwrap();
        let mut out_file = File::create(output_file_path)?;
        out_file.write(vm.as_bytes())?;
        // print!("{}", xml);
    }

    Ok(())
}
