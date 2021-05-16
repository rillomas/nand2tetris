use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct IOSet {
	pub input: BufReader<std::fs::File>,
	pub input_file: PathBuf,
	pub output_file: PathBuf,
}

/// Read a file path or directory of files to get valid input/output file paths
pub fn generate_ioset(input_path: &Path) -> Result<Vec<IOSet>, std::io::Error> {
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
