use std::path::PathBuf;

#[test]
fn test_tokenized_xml() {
	let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	root.push("tests/data")
	let dirs = vec!["ArrayTest","ExpressionLessSquare", "Square"];
	for dir in dirs {
		// Convert jack to token xml for each directory
		// let tokens = token::generate_token_list(&mut file.input);
		// let xml = to_string(&tokens).unwrap();
	}
	// Read golden xml
	// Create my xml and compare

	// assert_eq!(adder::add(3, 2), 5);
}
