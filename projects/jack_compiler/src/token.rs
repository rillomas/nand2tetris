/// Context of the file parsing process
pub struct FileContext {
	/// Whether current line started as a multiline comment
	in_comment: bool,
}

impl FileContext {
	pub fn new() -> FileContext {
		FileContext { in_comment: false }
	}
}

#[derive(Debug, Copy, Clone)]
pub enum TokenType {
	Keyword,
	Symbol,
	Identifier,
	IntegerConst,
	StringConst,
}

#[derive(Debug, Copy, Clone)]
pub enum KeywordType {
	Class,
	Method,
	Function,
	Constructor,
	Int,
	Boolean,
	Char,
	Void,
	Var,
	Static,
	Field,
	Let,
	Do,
	If,
	Else,
	While,
	Return,
	True,
	False,
	Null,
	This,
}

pub trait Token: std::fmt::Debug {
	fn r#type(&self) -> TokenType;
}

struct Symbol {
	r#type: TokenType,
	symbol: char,
}

struct Identifier {
	r#type: TokenType,
	symbol: char,
}

/// Current context within a line
struct LineContext {}

pub fn parse_line(context: &mut FileContext, line: &str) -> Vec<Box<dyn Token>> {
	let out: Vec<Box<dyn Token>> = Vec::new();
	let mut in_comment = context.in_comment;
	let mut finished_parsing = false;
	let line_context = LineContext {};
	while finished_parsing {
		// Try to obtain next token by reading the next character

		if in_comment {
			// Since we are in a multi line comment we just look for the closing comment symbol.
			// If we cannot find it we don't add any tokens.
			// If we find it we remove all the comment part and process the rest
		} else {
			// We look for
		}
		// We still have valid token area left over so we parse it
	}
	out
}
