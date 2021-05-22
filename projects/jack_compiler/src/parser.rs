use super::tokenizer;

pub struct Tree {}

impl Tree {
	pub fn serialize(&self) -> Result<String, String> {
		let xml = String::from("");
		Ok(xml)
	}
}

fn parse_token_list(_tokens: &tokenizer::TokenList) -> Tree {
	Tree {}
}

/// Generate parsed tree from given file reader
pub fn generate_tree(file_reader: &mut std::io::BufReader<std::fs::File>) -> Tree {
	let tokens = tokenizer::generate_token_list(file_reader);
	parse_token_list(&tokens)
}
