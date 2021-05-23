use super::tokenizer::{generate_token_list, TokenList, TokenType};

pub struct Tree {}

impl Tree {
	pub fn serialize(&self) -> Result<String, String> {
		let xml = String::from("");
		Ok(xml)
	}
}

struct Class {}

/// Convert token list to a compiled tree
fn parse_token_list(tokens: &TokenList) -> Tree {
	for t in tokens.iter() {
		match t.token() {
			TokenType::Keyword => {}
			TokenType::Symbol => {}
			TokenType::Identifier => {}
			TokenType::IntegerConst => {}
			TokenType::StringConst => {}
		}
	}
	Tree {}
}

/// Generate parsed tree from given file reader
pub fn generate_tree(file_reader: &mut std::io::BufReader<std::fs::File>) -> Tree {
	let tokens = generate_token_list(file_reader);
	parse_token_list(&tokens)
}
