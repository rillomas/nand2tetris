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

#[derive(Debug)]
struct Symbol {
	symbol: char,
}

impl Token for Symbol {
	fn r#type(&self) -> TokenType {
		TokenType::Symbol
	}
}

#[derive(Debug)]
struct Identifier {
	sequence: String,
}

impl Token for Identifier {
	fn r#type(&self) -> TokenType {
		TokenType::Identifier
	}
}

#[derive(Debug)]
struct IntegerConstant {
	value: u16,
}

impl Token for IntegerConstant {
	fn r#type(&self) -> TokenType {
		TokenType::IntegerConst
	}
}

#[derive(Debug)]
struct StringConstant {
	sequence: String,
}

impl Token for StringConstant {
	fn r#type(&self) -> TokenType {
		TokenType::StringConst
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
}

const ASTERISK: char = '*';
const SLASH: char = '/';

enum LineParseResult {
	/// End parsing line and go to next line
	Break,
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
					return LineParseResult::Break;
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

pub fn parse_line(context: &mut FileContext, line: &str) -> Vec<Box<dyn Token>> {
	let out: Vec<Box<dyn Token>> = Vec::new();
	let mut ctx = LineContext {
		comment: CommentState {
			in_region: context.in_comment,
			next_maybe_line_begin: false,
			next_maybe_region_begin: false,
			next_maybe_region_end: false,
		},
	};
	// iterate over all character
	for c in line.chars() {
		let ret = update_comment_state(&mut ctx.comment, c);
		println!("{} {:?}", c, ctx.comment);
		if matches!(ret, LineParseResult::Break) {
			break;
		}
		// If it is an alphabet we read until next symbol (except underscore) or space
		// If it is a '/' we check the next symbol to see if it is a comment symbol
		// If it is a valid symbol

		// if ctx.in_comment {
		// 	// Since we are in a multi line comment we just look for the closing comment symbol.
		// 	// If we cannot find it we don't add any tokens.
		// 	// If we find it we remove all the comment part and process the rest
		// } else {
		// 	// We look for
		// }
	}
	// update context for the next line
	context.in_comment = ctx.comment.in_region;
	out
}
