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

pub fn parse_line(line: &str) -> Vec<Box<dyn Token>> {
	let out: Vec<Box<dyn Token>> = Vec::new();
	out
}
