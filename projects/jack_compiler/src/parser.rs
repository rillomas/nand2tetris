use super::tokenizer::{
	generate_token_list, Identifier, IntegerConstant, Keyword, KeywordType, StringConstant, Symbol,
	TokenList, TokenType,
};

const INDENT_STR: &'static str = "  ";

pub trait Node {
	/// Serialize node at the specified indent level
	fn serialize(&self, output: &mut String, indent_level: usize);
	/// Add child node
	fn add(&mut self, node: Box<dyn Node>);
}

struct Root {
	nodes: Vec<Box<dyn Node>>,
}

impl Node for Root {
	fn serialize(&self, output: &mut String, indent_level: usize) {
		for n in &self.nodes {
			// Root class is the root so all nodes are at level 0 indent
			n.serialize(output, 0);
		}
	}

	fn add(&mut self, node: Box<dyn Node>) {
		self.nodes.push(node)
	}
}

struct Class {}

impl Node for Class {
	fn serialize(&self, output: &mut String, indent_level: usize) {
		let label = "class";
		let indent = INDENT_STR.repeat(indent_level);
		let start_tag = format!("{0}<{1}>\r\n", indent, label);
		let end_tag = format!("{0}</{1}>\r\n", indent, label);
		output.push_str(&start_tag);
		output.push_str(&end_tag);
	}
	fn add(&mut self, node: Box<dyn Node>) {}
}

/// Check and ingest all tokens related to current class
fn compile_class(
	parent: &mut dyn Node,
	tokens: &TokenList,
	token_index: usize,
) -> Result<usize, String> {
	// Check tokens from the head to see if they are valid class tokens
	// If it seems valid we add class to the tree
	parent.add(Box::new(Class {}));
	Ok(tokens.list.len())
}

/// Convert token list to a compiled tree
fn parse_token_list(
	parent: &mut dyn Node,
	tokens: &TokenList,
	token_index: usize,
) -> Result<usize, String> {
	let mut current_index = token_index;
	loop {
		if current_index >= tokens.list.len() {
			// Exit when we finished processing all tokens
			break;
		}
		let t = &tokens.list[current_index];
		match t.token() {
			TokenType::Keyword => {
				let keyword = t.as_any().downcast_ref::<Keyword>().unwrap();
				// println!("keyword: {:?}", keyword);
				match keyword.keyword() {
					KeywordType::Class => {
						current_index = compile_class(parent, tokens, current_index)?;
					}
					_other => {}
				}
			}
			TokenType::Symbol => {}
			TokenType::Identifier => {}
			TokenType::IntegerConst => {}
			TokenType::StringConst => {}
		}
	}
	Ok(current_index)
}

/// Generate parsed tree from given file reader
pub fn generate_tree(
	file_reader: &mut std::io::BufReader<std::fs::File>,
) -> Result<Box<dyn Node>, String> {
	let tokens = generate_token_list(file_reader);
	let mut root = Root { nodes: Vec::new() };
	parse_token_list(&mut root, &tokens, 0)?;
	Ok(Box::new(root))
}
