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
struct Keyword {
	sequence: String,
}

impl Token for Keyword {
	fn r#type(&self) -> TokenType {
		TokenType::Keyword
	}
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
	/// True if current char is inside a string constant
	in_string: bool,
	/// List of chars that are not yet finished as a token
	char_stash: Vec<char>,
}

const ASTERISK: char = '*';
const SLASH: char = '/';
const DOUBLE_QUOTE: char = '"';
const SYMBOL_LIST: [char; 19] = [
	'}', '{', ')', '(', '[', ']', '.', ',', ';', '+', '-', '*', SLASH, '&', '|', '<', '>', '=', '~',
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
		Ok(Box::new(Symbol { symbol: stash[0] }))
	} else if stash[0].is_ascii_digit() {
		// If the first symbol is an integer it is an integer const
		Ok(Box::new(IntegerConstant {
			value: str::parse::<u16>(&word.as_str()).unwrap(),
		}))
	} else if KEYWORD_LIST.contains(&word.as_str()) {
		// If the word matches keyword list we return keyword
		Ok(Box::new(Keyword { sequence: word }))
	} else {
		// all other cases are identifiers
		Ok(Box::new(Identifier { sequence: word }))
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
				token_list.push(Box::new(StringConstant { sequence: str }));
				ctx.char_stash.clear();
				ctx.in_string = false;
			} else {
				ctx.char_stash.push(c);
			}
		} else {
			// not in string
			let ret = update_comment_state(&mut ctx.comment, c);
			if matches!(ret, LineParseResult::LineComment) {
				// We encountered a line comment symbol so we break here and go to next line.
				// left over token should be the previous '/' symbol so we just drop it and go on
				break;
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
						token_list.push(Box::new(Symbol { symbol: c }));
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
