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
    // Gather information from all files
    let mut dir_info = jack_compiler::parser::DirectoryParseInfo::new();
    let mut class_list = Vec::new();
    for mut io in io_list {
        println!("input: {}", &io.input_file.display());
        let mut output_file_path = io.input_file.clone();
        let origin_name = jack_compiler::get_origin_name(&io.input_file).unwrap();
        let out_name = format!("{}.vm", origin_name);
        output_file_path.set_file_name(out_name);
        let mut info = jack_compiler::parser::ClassParseInfo::new();
        let class = jack_compiler::parser::parse_file(&mut info, &mut io.input).unwrap();
        dir_info
            .info_per_class
            .insert(class.name().to_owned(), info);
        class_list.push((class, output_file_path));
    }

    // compile all files
    for (c, out_path) in class_list {
        println!("output: {}", &out_path.display());
        let vm = c.compile(&dir_info).unwrap();
        let mut out_file = File::create(out_path)?;
        out_file.write(vm.as_bytes())?;
        // print!("{}", xml);
    }
    Ok(())
}
