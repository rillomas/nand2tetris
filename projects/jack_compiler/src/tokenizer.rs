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

pub const NEW_LINE: &str = "\r\n";
pub const INDENT_STR: &'static str = "  ";
#[derive(thiserror::Error, Debug)]
pub enum SerializeError {
    #[error("Unexpected State: {0}")]
    UnexpectedState(String),
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

/// Generate token list from given file reader
pub fn generate_token_list(file_reader: &mut std::io::BufReader<std::fs::File>) -> TokenList {
    let mut tokens = TokenList { list: Vec::new() };
    let mut context = FileContext::new();
    for line in file_reader.lines() {
        let line_text = line.unwrap();
        let mut tk = parse_line(&mut context, &line_text);
        tokens.list.append(&mut tk);
    }
    tokens
}

#[derive(Debug)]
pub struct TokenList {
    pub list: Vec<Token>,
}

impl TokenList {
    /// Serialize each token to XML
    pub fn serialize(&self) -> Result<String, SerializeError> {
        let mut output = String::new();
        let tag = "tokens";
        let start_tag = format!("<{0}>{1}", tag, NEW_LINE);
        let end_tag = format!("</{0}>{1}", tag, NEW_LINE);
        output.push_str(&start_tag);
        for e in &self.list {
            e.serialize(&mut output, 0)?;
        }
        output.push_str(&end_tag);
        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(Identifier),
    IntegerConstant(IntegerConstant),
    StringConstant(StringConstant),
}

impl Token {
    /// Serialize each token to XML
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        match self {
            Token::Keyword(k) => k.serialize(output, indent_level),
            Token::Symbol(s) => s.serialize(output, indent_level),
            Token::Identifier(i) => i.serialize(output, indent_level),
            Token::IntegerConstant(ic) => ic.serialize(output, indent_level),
            Token::StringConstant(sc) => sc.serialize(output, indent_level),
        }
    }

    /// Get a string representation of token
    pub fn string(&self) -> String {
        match self {
            Token::Keyword(k) => k.string(),
            Token::Symbol(s) => s.string(),
            Token::Identifier(i) => i.string(),
            Token::IntegerConstant(ic) => ic.string(),
            Token::StringConstant(sc) => sc.string(),
        }
    }

    pub fn symbol(&self) -> Option<&Symbol> {
        match self {
            Token::Symbol(s) => Some(s),
            _ => None,
        }
    }
    pub fn identifier(&self) -> Option<&Identifier> {
        match self {
            Token::Identifier(i) => Some(i),
            _ => None,
        }
    }
    pub fn keyword(&self) -> Option<&Keyword> {
        match self {
            Token::Keyword(k) => Some(k),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keyword {
    pub value: String,
}

const STATIC: &str = "static";
const CONSTRUCTOR: &str = "constructor";
const FUNCTION: &str = "function";
pub const CLASS: &str = "class";
const METHOD: &str = "method";
const FIELD: &str = "field";
const VOID: &str = "void";
const INT: &str = "int";
const CHAR: &str = "char";
const BOOL: &str = "boolean";
const VAR: &str = "var";
const LET: &str = "let";
const IF: &str = "if";
const ELSE: &str = "else";
const WHILE: &str = "while";
const DO: &str = "do";
const RETURN: &str = "return";
const THIS: &str = "this";
const TRUE: &str = "true";
const FALSE: &str = "false";
const NULL: &str = "null";

impl Keyword {
    pub fn new() -> Keyword {
        Keyword {
            value: String::new(),
        }
    }

    pub fn keyword(&self) -> KeywordType {
        match self.value.as_str() {
            CLASS => KeywordType::Class,
            CONSTRUCTOR => KeywordType::Constructor,
            FUNCTION => KeywordType::Function,
            METHOD => KeywordType::Method,
            FIELD => KeywordType::Field,
            STATIC => KeywordType::Static,
            VAR => KeywordType::Var,
            INT => KeywordType::Int,
            CHAR => KeywordType::Char,
            BOOL => KeywordType::Boolean,
            VOID => KeywordType::Void,
            TRUE => KeywordType::True,
            FALSE => KeywordType::False,
            NULL => KeywordType::Null,
            THIS => KeywordType::This,
            LET => KeywordType::Let,
            DO => KeywordType::Do,
            IF => KeywordType::If,
            ELSE => KeywordType::Else,
            WHILE => KeywordType::While,
            RETURN => KeywordType::Return,
            _ => panic!("Unknowon keyword"),
        }
    }
}
impl Keyword {
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let tag = "keyword";
        let indent = INDENT_STR.repeat(indent_level);
        let str = format!("{0}<{1}> {2} </{1}>{3}", indent, tag, self.value, NEW_LINE);
        output.push_str(&str);
        Ok(())
    }

    pub fn string(&self) -> String {
        self.value.to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub value: char,
}

impl Symbol {
    pub fn new() -> Symbol {
        Symbol {
            value: '\0', // Init with a null character
        }
    }
}

fn escape_char(c: &char) -> String {
    match c {
        '<' => String::from("&lt;"),
        '>' => String::from("&gt;"),
        '"' => String::from("&quot;"),
        '\'' => String::from("&apos;"),
        '&' => String::from("&amp;"),
        _other => _other.to_string(),
    }
}

impl Symbol {
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let tag = "symbol";
        let escaped = escape_char(&self.value);
        let indent = INDENT_STR.repeat(indent_level);
        let str = format!("{0}<{1}> {2} </{1}>{3}", indent, tag, escaped, NEW_LINE);
        output.push_str(&str);
        Ok(())
    }

    pub fn string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    pub fn new() -> Identifier {
        Identifier {
            value: String::new(),
        }
    }
}

impl Identifier {
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let tag = "identifier";
        let indent = INDENT_STR.repeat(indent_level);
        let str = format!("{0}<{1}> {2} </{1}>{3}", indent, tag, self.value, NEW_LINE);
        output.push_str(&str);
        Ok(())
    }

    pub fn string(&self) -> String {
        self.value.to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerConstant {
    value: u16,
}

impl IntegerConstant {
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let tag = "integerConstant";
        let indent = INDENT_STR.repeat(indent_level);
        let str = format!("{0}<{1}> {2} </{1}>{3}", indent, tag, self.value, NEW_LINE);
        output.push_str(&str);
        Ok(())
    }

    pub fn string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct StringConstant {
    value: String,
}

impl StringConstant {
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let tag = "stringConstant";
        let indent = INDENT_STR.repeat(indent_level);
        let str = format!("{0}<{1}> {2} </{1}>{3}", indent, tag, self.value, NEW_LINE);
        output.push_str(&str);
        Ok(())
    }

    pub fn string(&self) -> String {
        self.value.to_owned()
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

const SYMBOL_LIST: [char; 19] = [
    '}', '{', ')', '(', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=', '~',
];

const KEYWORD_LIST: [&str; 21] = [
    CLASS,
    CONSTRUCTOR,
    FUNCTION,
    METHOD,
    FIELD,
    STATIC,
    VAR,
    INT,
    CHAR,
    BOOL,
    VOID,
    TRUE,
    FALSE,
    NULL,
    THIS,
    LET,
    DO,
    IF,
    ELSE,
    WHILE,
    RETURN,
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
            '/' => {
                if state.next_maybe_region_end {
                    // We have reached end of region comment
                    state.in_region = false;
                    state.next_maybe_region_end = false;
                }
            }
            '*' => {
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
            '/' => {
                if state.next_maybe_line_begin {
                    // line comment has begun so we skip the rest
                    return LineParseResult::LineComment;
                } else {
                    // Region comment or line comment may begin on next char
                    state.next_maybe_line_begin = true;
                    state.next_maybe_region_begin = true;
                }
            }
            '*' => {
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
fn extract_token(stash: &Vec<char>) -> Result<Token, &str> {
    let len = stash.len();
    if len == 0 {
        return Err("Empty stash given");
    }
    let word: String = stash.iter().cloned().collect();

    if len == 1 && SYMBOL_LIST.contains(&stash[0]) {
        // Got a symbol
        Ok(Token::Symbol(Symbol { value: stash[0] }))
    } else if stash[0].is_ascii_digit() {
        // If the first symbol is an integer it is an integer const
        Ok(Token::IntegerConstant(IntegerConstant {
            value: str::parse::<u16>(&word.as_str()).unwrap(),
        }))
    } else if KEYWORD_LIST.contains(&word.as_str()) {
        // If the word matches keyword list we return keyword
        Ok(Token::Keyword(Keyword { value: word }))
    } else {
        // all other cases are identifiers
        Ok(Token::Identifier(Identifier { value: word }))
    }
}

pub fn parse_line(context: &mut FileContext, line: &str) -> Vec<Token> {
    let mut token_list = Vec::new();
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
            if c == '"' {
                // We are now at end of string
                // Get all stashed characters and push to token list
                let str = ctx.char_stash.iter().collect();
                token_list.push(Token::StringConstant(StringConstant { value: str }));
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
            } else if c == '"' {
                // We are at start of string
                ctx.in_string = true;
            } else if SYMBOL_LIST.contains(&c) {
                // Got a symbol
                match c {
                    '/' => {
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
                        token_list.push(Token::Symbol(Symbol { value: c }));
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
