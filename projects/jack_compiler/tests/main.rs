use std::path::PathBuf;
// mod jack_compiler;
// use super::*;

#[test]
fn test_tokenized_xml() -> Result<(), std::io::Error> {
	let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let dirs = vec!["ArrayTest", "ExpressionLessSquare", "Square"];
	for dir in dirs {
		let target = root.join("test/data").join(dir);
		// Convert jack to token xml for each directory
		let io_list = jack_compiler::generate_ioset(&target)?;
		println!("{:?}", io_list)
		// let tokens = generate_token_list(&mut file.input);
		// let xml = to_string(&tokens).unwrap();
	}
	Ok(())
	// Read golden xml
	// Create my xml and compare

	// assert_eq!(adder::add(3, 2), 5);
}
