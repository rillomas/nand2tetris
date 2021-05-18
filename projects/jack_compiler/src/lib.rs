use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
pub mod token;

#[derive(Debug)]
pub struct IOSet {
	pub input: BufReader<std::fs::File>,
	pub input_file: PathBuf,
}

/// Get the origin name (file stem) of a given path
pub fn get_origin_name(input_path: &Path) -> Result<String, std::ffi::OsString> {
	input_path.file_stem().unwrap().to_os_string().into_string()
}

/// Read a file path or directory of files to get valid input/output file paths
pub fn generate_ioset(input_path: &Path) -> Result<Vec<IOSet>, std::io::Error> {
	let mut file_list = Vec::new();
	if input_path.is_file() {
		// load single file by single reader
		let file = File::open(input_path)?;
		let set = IOSet {
			input: BufReader::new(file),
			input_file: input_path.to_owned(),
		};
		file_list.push(set);
		Ok(file_list)
	} else if input_path.is_dir() {
		// load all files by multiple reader
		for entry in std::fs::read_dir(input_path)? {
			let path = entry.unwrap().path();
			if path.extension().unwrap() == "jack" {
				// only look at vm files
				let file = File::open(&path)?;
				let set = IOSet {
					input: BufReader::new(file),
					input_file: path.to_owned(),
				};
				file_list.push(set);
			}
		}
		Ok(file_list)
	} else {
		panic!("Unsupported path specified");
	}
}
