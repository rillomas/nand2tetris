use jack_compiler::{generate_ioset, get_origin_name, token};
// use quick_xml::de::from_reader;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

const TEST_DIR: &str = "tests";
const DATA_DIR: &str = "data";

#[test]
fn test_tokenized_xml() -> Result<(), std::io::Error> {
	let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let dirs = vec!["ArrayTest", "ExpressionLessSquare", "Square"];
	for dir in dirs {
		let target = root.join(TEST_DIR).join(DATA_DIR).join(dir);
		// println!("{:?}", target);
		// Convert jack to token xml for each directory
		let io_list = generate_ioset(&target)?;
		// println!("{:?}", io_list);
		for mut io in io_list {
			let origin = get_origin_name(&io.input_file).unwrap();
			let mut golden_file_path = io.input_file.clone();
			let golden_name = format!("{}T.xml", origin);
			golden_file_path.set_file_name(golden_name);
			let tokens = token::generate_token_list(&mut io.input);

			// Generate XML from golden results
			// let golden = File::open(golden_file_path).unwrap();
			// let reader = BufReader::new(golden);
			// let list: token::TokenList = from_reader(reader).unwrap();

			// let xml = to_string(&tokens).unwrap();
		}
	}
	Ok(())
	// Read golden xml
	// Create my xml and compare

	// assert_eq!(adder::add(3, 2), 5);
}
