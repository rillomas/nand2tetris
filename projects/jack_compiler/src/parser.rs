use super::tokenizer::{
	generate_token_list, Identifier, IntegerConstant, Keyword, KeywordType, StringConstant, Symbol,
	Token, TokenList, TokenType,
};

const INDENT_STR: &'static str = "  ";

pub trait Node {
	/// Serialize node at the specified indent level
	fn serialize(&self, output: &mut String, indent_level: usize);
	/// Add child node
	fn add_child(&mut self, node: Box<dyn Node>);
}

struct Root {
	nodes: Vec<Box<dyn Node>>,
}

impl Node for Root {
	fn serialize(&self, output: &mut String, _indent_level: usize) {
		for n in &self.nodes {
			// Root class is the root so all nodes are at level 0 indent
			n.serialize(output, 0);
		}
	}

	fn add_child(&mut self, node: Box<dyn Node>) {
		self.nodes.push(node)
	}
}

struct Class {
	name: Identifier,
	begin_symbol: Symbol,
	end_symbol: Symbol,
	nodes: Vec<Box<dyn Node>>,
}

impl Class {
	fn new() -> Class {
		Class {
			name: Identifier::new(),
			begin_symbol: Symbol::new(),
			end_symbol: Symbol::new(),
			nodes: Vec::new(),
		}
	}
}

impl Node for Class {
	fn serialize(&self, output: &mut String, indent_level: usize) {
		let label = "class";
		let indent = INDENT_STR.repeat(indent_level);
		let start_tag = format!("{0}<{1}>\r\n", indent, label);
		let end_tag = format!("{0}</{1}>\r\n", indent, label);
		output.push_str(&start_tag);
		self.name.serialize(output);
		self.begin_symbol.serialize(output);
		let next_level = indent_level + 1;
		for c in &self.nodes {
			c.serialize(output, next_level);
		}
		self.end_symbol.serialize(output);
		output.push_str(&end_tag);
	}
	fn add_child(&mut self, node: Box<dyn Node>) {}
}

fn compile_identifier(token: &Box<dyn Token>) -> Result<&Identifier, String> {
	if !matches!(token.token(), TokenType::Identifier) {
		return Err(String::from("Expected Identifier token"));
	}
	let id = token.as_any().downcast_ref::<Identifier>().unwrap();
	Ok(id)
}

/// Check and ingest all tokens related to current class
fn compile_class(
	parent: &mut dyn Node,
	tokens: &TokenList,
	token_index: usize,
) -> Result<usize, String> {
	// Check tokens from the head to see if they are valid class tokens
	let mut class = Box::new(Class::new());
	let mut current_idx = token_index;
	let name_token = &tokens.list[current_idx];
	let name = compile_identifier(name_token)?;
	// TODO: store name in type table
	class.name = name.clone();
	current_idx += 1;
	let brace_token = &tokens.list[current_idx];
	if !matches!(brace_token.token(), TokenType::Symbol) {
		return Err(String::from("Expected Symbol token for class"));
	}
	let brace_symbol = brace_token.as_any().downcast_ref::<Symbol>().unwrap();
	if !(brace_symbol.value == '{') {
		return Err(String::from("Expected open bracket for class"));
	}
	class.begin_symbol = brace_symbol.clone();
	// If it seems valid we add class to the tree
	parent.add_child(class);
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
		current_index += 1;
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
