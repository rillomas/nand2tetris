use std::io::BufRead;

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
const TOKEN_NEW_LINE: &str = "\r\n";

// #[derive(Debug, Copy, Clone)]
// pub enum KeywordType {
// 	Class,
// 	Method,
// 	Function,
// 	Constructor,
// 	Int,
// 	Boolean,
// 	Char,
// 	Void,
// 	Var,
// 	Static,
// 	Field,
// 	Let,
// 	Do,
// 	If,
// 	Else,
// 	While,
// 	Return,
// 	True,
// 	False,
// 	Null,
// 	This,
// }

/// Generate token list from given file reader
pub fn generate_token_list(file_reader: &mut std::io::BufReader<std::fs::File>) -> TokenList {
	let mut tokens = TokenList(Vec::new());
	let mut context = FileContext::new();
	for line in file_reader.lines() {
		let line_text = line.unwrap();
		let mut tk = parse_line(&mut context, &line_text);
		tokens.0.append(&mut tk);
	}
	tokens
}

#[derive(Debug)]
pub struct TokenList(Vec<Box<dyn Token>>);

impl TokenList {
	/// Serialize each token to XML
	pub fn serialize(&self) -> Result<String, String> {
		let mut output = String::new();
		let tag = "tokens";
		let start_tag = format!("<{0}>{1}", tag, TOKEN_NEW_LINE);
		output.push_str(&start_tag);
		for e in &self.0 {
			e.serialize(&mut output);
		}
		let end_tag = format!("</{0}>{1}", tag, TOKEN_NEW_LINE);
		output.push_str(&end_tag);
		Ok(output)
	}
}

pub trait Token: std::fmt::Debug {
	fn r#type(&self) -> TokenType;
	/// Serialize each token to XML
	fn serialize(&self, output: &mut String);
}

#[derive(Debug)]
struct Keyword(String);

impl Token for Keyword {
	fn r#type(&self) -> TokenType {
		TokenType::Keyword
	}

	fn serialize(&self, output: &mut String) {
		let tag = "keyword";
		let str = format!("<{0}> {1} </{0}>{2}", tag, self.0, TOKEN_NEW_LINE);
		output.push_str(&str);
	}
}

#[derive(Debug)]
struct Symbol(char);

impl Token for Symbol {
	fn r#type(&self) -> TokenType {
		TokenType::Symbol
	}
	fn serialize(&self, output: &mut String) {
		let tag = "symbol";
		let str = format!("<{0}> {1} </{0}>{2}", tag, self.0, TOKEN_NEW_LINE);
		output.push_str(&str);
	}
}

#[derive(Debug)]
struct Identifier(String);

impl Token for Identifier {
	fn r#type(&self) -> TokenType {
		TokenType::Identifier
	}
	fn serialize(&self, output: &mut String) {
		let tag = "identifier";
		let str = format!("<{0}> {1} </{0}>{2}", tag, self.0, TOKEN_NEW_LINE);
		output.push_str(&str);
	}
}

#[derive(Debug)]
struct IntegerConstant(u16);

impl Token for IntegerConstant {
	fn r#type(&self) -> TokenType {
		TokenType::IntegerConst
	}

	fn serialize(&self, output: &mut String) {
		let tag = "integerConst";
		let str = format!("<{0}> {1} </{0}>{2}", tag, self.0, TOKEN_NEW_LINE);
		output.push_str(&str);
	}
}

#[derive(Debug)]
struct StringConstant(String);

impl Token for StringConstant {
	fn r#type(&self) -> TokenType {
		TokenType::StringConst
	}

	fn serialize(&self, output: &mut String) {
		let tag = "stringConst";
		let str = format!("<{0}> {1} </{0}>{2}", tag, self.0, TOKEN_NEW_LINE);
		output.push_str(&str);
	}
}

/// State to manage comment situation
#[derive(Debug)]
struct CommentState {
	/// Current character is in block comment region
	in_region: bool,
	/// Next character maybe line comment begin ('//')
	next_maybe_line_begin: bool,
	/// Next character maybe region comment begin ('/*')
	next_maybe_region_begin: bool,
	/// Next character maybe region comment end ('*/')
	next_maybe_region_end: bool,
}
/// Current context within a line
struct LineContext {
	comment: CommentState,
	/// True if current char is inside a string constant
	in_string: bool,
	/// List of chars that are not yet finished as a token
	char_stash: Vec<char>,
}

const ASTERISK: char = '*';
const SLASH: char = '/';
const DOUBLE_QUOTE: char = '"';
const SYMBOL_LIST: [char; 19] = [
	'}', '{', ')', '(', '[', ']', '.', ',', ';', '+', '-', ASTERISK, SLASH, '&', '|', '<', '>', '=',
	'~',
];
const KEYWORD_LIST: [&str; 21] = [
	"class",
	"constructor",
	"function",
	"method",
	"field",
	"static",
	"var",
	"int",
	"char",
	"boolean",
	"void",
	"true",
	"false",
	"null",
	"this",
	"let",
	"do",
	"if",
	"else",
	"while",
	"return",
];

#[derive(Debug)]
enum LineParseResult {
	/// Got a line comment
	LineComment,
	/// Continue parsing line
	Continue,
}

/// Update comment state depending on character
fn update_comment_state(state: &mut CommentState, c: char) -> LineParseResult {
	if state.in_region {
		// In comment region
		match c {
			SLASH => {
				if state.next_maybe_region_end {
					// We have reached end of region comment
					state.in_region = false;
					state.next_maybe_region_end = false;
				}
			}
			ASTERISK => {
				if !state.next_maybe_region_end {
					// If we get a slash for next char comment region will end
					state.next_maybe_region_end = true;
				}
			}
			_ => {
				// For all other chars we reset region end flag
				state.next_maybe_region_end = false;
			}
		}
	} else {
		// Not in comment region
		match c {
			SLASH => {
				if state.next_maybe_line_begin {
					// line comment has begun so we skip the rest
					return LineParseResult::LineComment;
				} else {
					// Region comment or line comment may begin on next char
					state.next_maybe_line_begin = true;
					state.next_maybe_region_begin = true;
				}
			}
			ASTERISK => {
				if state.next_maybe_region_begin {
					// region comment has begun
					state.in_region = true;
					state.next_maybe_line_begin = false;
					state.next_maybe_region_begin = false;
				}
			}
			_ => {
				// For all other chars we reset comment state
				state.next_maybe_line_begin = false;
				state.next_maybe_region_begin = false;
			}
		}
	}
	LineParseResult::Continue
}

/// Create token by analyzing the content
fn extract_token(stash: &Vec<char>) -> Result<Box<dyn Token>, &str> {
	let len = stash.len();
	if len == 0 {
		return Err("Empty stash given");
	}
	let word: String = stash.iter().cloned().collect();

	if len == 1 && SYMBOL_LIST.contains(&stash[0]) {
		// Got a symbol
		Ok(Box::new(Symbol(stash[0])))
	} else if stash[0].is_ascii_digit() {
		// If the first symbol is an integer it is an integer const
		Ok(Box::new(IntegerConstant(
			str::parse::<u16>(&word.as_str()).unwrap(),
		)))
	} else if KEYWORD_LIST.contains(&word.as_str()) {
		// If the word matches keyword list we return keyword
		Ok(Box::new(Keyword(word)))
	} else {
		// all other cases are identifiers
		Ok(Box::new(Identifier(word)))
	}
}

pub fn parse_line(context: &mut FileContext, line: &str) -> Vec<Box<dyn Token>> {
	let mut token_list: Vec<Box<dyn Token>> = Vec::new();
	let mut ctx = LineContext {
		comment: CommentState {
			in_region: context.in_comment,
			next_maybe_line_begin: false,
			next_maybe_region_begin: false,
			next_maybe_region_end: false,
		},
		in_string: false,
		char_stash: Vec::new(),
	};
	// iterate over all character
	for c in line.chars() {
		// println!("{}", c);
		if ctx.in_string {
			// We are currently in a string so we stash all chars unless we get the end quote
			if c == DOUBLE_QUOTE {
				// We are now at end of string
				// Get all stashed characters and push to token list
				let str = ctx.char_stash.iter().collect();
				token_list.push(Box::new(StringConstant(str)));
				ctx.char_stash.clear();
				ctx.in_string = false;
			} else {
				ctx.char_stash.push(c);
			}
		} else {
			// not in string
			let ret = update_comment_state(&mut ctx.comment, c);
			match ret {
				LineParseResult::LineComment => {
					// We encountered a line comment symbol so we break here and go to next line.
					// left over token should be the previous '/' symbol so we just drop it and go on
					break;
				}
				LineParseResult::Continue => {
					// We just continue
				}
			}
			if ctx.comment.in_region {
				// We are in region comment so we go to next char
				// If we have any previous char it should be a '/' symbol so we drop it
				ctx.char_stash.clear();
				continue;
			}
			if c.is_whitespace() {
				// look at stash and if we have anything push it as token
				if !ctx.char_stash.is_empty() {
					token_list.push(extract_token(&ctx.char_stash).unwrap());
					ctx.char_stash.clear();
				}
			} else if c == DOUBLE_QUOTE {
				// We are at start of string
				ctx.in_string = true;
			} else if SYMBOL_LIST.contains(&c) {
				// Got a symbol
				match c {
					SLASH => {
						// May be a div symbol or comment symbol.
						// We stash the character and go next
						ctx.char_stash.push(c);
						continue;
					}
					_ => {
						// All other symbols can be simply added as token
						// If we already have anything in the stash we push it as a token first
						if !ctx.char_stash.is_empty() {
							token_list.push(extract_token(&ctx.char_stash).unwrap());
							ctx.char_stash.clear();
						}
						token_list.push(Box::new(Symbol(c)));
					}
				}
			} else {
				// Push all other char to stash
				ctx.char_stash.push(c);
			}
		}
	}
	// update context for the next line
	context.in_comment = ctx.comment.in_region;
	token_list
}
