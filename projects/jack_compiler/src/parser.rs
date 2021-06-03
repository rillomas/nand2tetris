use super::tokenizer;
use super::tokenizer::{
    generate_token_list, Identifier, Keyword, KeywordType, Symbol, Token, TokenList, TokenType,
    INDENT_STR, NEW_LINE,
};

const CLASS_VAR_DEC: &'static str = "classVarDec";
const SUBROUTINE_DEC: &'static str = "subroutineDec";
type SerializeError = String;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got unexpected token type: {0:?}")]
    UnexpectedToken(TokenType),
    #[error("Got unexpected keyword: {0}")]
    UnexpectedKeyword(String),
    #[error("Got unknown type: {0}")]
    UnknownType(String),
    #[error("Got unexpected symbol: {0}")]
    UnexpectedSymbol(char),
}

pub trait Node {
    /// Serialize node at the specified indent level
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError>;
}

struct Context {
    /// Names of all user defined classes.
    /// Used to resolve types
    class_names: Vec<String>,
}

impl Context {
    fn new() -> Context {
        Context {
            class_names: Vec::new(),
        }
    }
}

struct Class {
    prefix: Keyword,
    name: Identifier,
    begin_symbol: Symbol,
    end_symbol: Symbol,
    children: Vec<Box<dyn Node>>,
}

impl Class {
    fn new() -> Class {
        Class {
            prefix: Keyword::new(),
            name: Identifier::new(),
            begin_symbol: Symbol::new(),
            end_symbol: Symbol::new(),
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), Error> {
        Ok(self.children.push(node))
    }
}

impl Node for Class {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = tokenizer::CLASS;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.prefix.serialize(output, next_level)?;
        self.name.serialize(output, next_level)?;
        self.begin_symbol.serialize(output, next_level)?;
        for c in &self.children {
            c.serialize(output, next_level)?;
        }
        self.end_symbol.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

struct ClassVarDec {
    prefix: Keyword,
    var_type: Box<dyn Token>, // var_type maybe a Keyword or an Identifier
    var_names: Vec<Identifier>,
    var_delimiter: Vec<Symbol>,
    end_symbol: Symbol,
}

impl ClassVarDec {
    fn new(prefix: Keyword) -> ClassVarDec {
        ClassVarDec {
            prefix: prefix,
            var_type: Box::new(Keyword::new()),
            var_names: Vec::new(),
            var_delimiter: Vec::new(),
            end_symbol: Symbol::new(),
        }
    }
}

impl Node for ClassVarDec {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        // number of vars and number of delimiters should match unless when we only have one var_name
        let var_num = self.var_names.len();
        let delim_num = self.var_delimiter.len();
        if var_num == 0 {
            return Err(String::from("Missing variable name"));
        } else if (var_num == 1) && (delim_num != 0) {
            return Err(String::from(
                "No delimiter should exist when we only have one variable",
            ));
        } else if (var_num > 1) && (delim_num != var_num) {
            return Err(String::from("Number of delimiter should match number of variables when there are multiple variables"));
        }
        let label = CLASS_VAR_DEC;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.prefix.serialize(output, next_level)?;
        // Serialize var type. Either a builtin type or a user class should be specified
        self.var_type.serialize(output, next_level)?;
        if var_num == 1 {
            // single variable
            self.var_names[0].serialize(output, next_level)?;
        } else {
            // multiple variables
            for i in 0..var_num {
                self.var_names[i].serialize(output, next_level)?;
                self.var_delimiter[i].serialize(output, next_level)?;
            }
        }
        self.end_symbol.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

struct SubroutineDec {
    prefix: Keyword,
    return_type: Box<dyn Token>, // var_type is a Keyword or an Identifier
    name: Identifier,
    start_param_list: Symbol,
    param_type: Vec<Box<dyn Token>>, // param_type is a Keyword or an Identifier
    param_name: Vec<Identifier>,
    param_delimiter: Vec<Symbol>,
    end_param_list: Symbol,
}

impl SubroutineDec {
    fn new(prefix: Keyword) -> SubroutineDec {
        SubroutineDec {
            prefix: prefix,
            return_type: Box::new(Keyword::new()),
            name: Identifier::new(),
            start_param_list: Symbol::new(),
            param_type: Vec::new(),
            param_name: Vec::new(),
            param_delimiter: Vec::new(),
            end_param_list: Symbol::new(),
        }
    }
}

impl Node for SubroutineDec {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = SUBROUTINE_DEC;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.prefix.serialize(output, next_level)?;
        self.return_type.serialize(output, next_level)?;
        self.name.serialize(output, next_level)?;
        self.start_param_list.serialize(output, next_level)?;
        self.end_param_list.serialize(output, next_level)?;
        // TODO: add subroutine body
        output.push_str(&end_tag);
        Ok(())
    }
}

fn compile_subroutinedec(
    ctx: &mut Context,
    target: &mut SubroutineDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.return_type = compile_return_type(ctx, &tokens.list[current_idx])?;
    current_idx += 1;
    target.name = compile_identifier(&tokens.list[current_idx])?.to_owned();
    current_idx += 1;
    let s = compile_symbol(&tokens.list[current_idx])?.to_owned();
    if s.value != '(' {
        return Err(Error::UnexpectedSymbol(s.value));
    }
    target.start_param_list = s;
    current_idx += 1;
    // compile param list
    loop {
        // compile param type and name once
        target
            .param_type
            .push(compile_type(ctx, &tokens.list[current_idx])?);
        current_idx += 1;
        target
            .param_name
            .push(compile_identifier(&tokens.list[current_idx])?.to_owned());
        current_idx += 1;
        break;
        // If we have a comma next we compile another round of params
        // If we have a semicolon param list is finished
        // let tk = &tokens.list[current_idx];
        // match tk.token() {
        //     TokenType::Symbol => {
        //         let s = tk.as_any().downcast_ref::<Symbol>().unwrap();
        //         match s.value {
        //             ',' => target.param_delimiter.push(s.to_owned()),
        //             ')' => {
        //                 // We got end of node symbol so we store it and go next
        //                 target.end_param_list = s.to_owned();
        //                 break;
        //             }
        //             _other => {
        //                 return Err(format!("Got unexpected symbol: {}", s.value));
        //             }
        //         }
        //     }
        //     TokenType::Identifier => {
        //         let i = tk.as_any().downcast_ref::<Identifier>().unwrap();
        //         target.var_names.push(i.to_owned());
        //     }
        //     _other => {
        //         return Err(format!("Got unexpected token type: {:?}", _other));
        //     }
        // }
    }
    Ok(current_idx)
}

fn compile_type(ctx: &mut Context, token: &Box<dyn Token>) -> Result<Box<dyn Token>, Error> {
    match token.token() {
        TokenType::Keyword => {
            let word = token.as_any().downcast_ref::<Keyword>().unwrap();
            match word.value.as_str() {
                tokenizer::INT | tokenizer::CHAR | tokenizer::BOOL => Ok(Box::new(word.to_owned())),
                _other => Err(Error::UnexpectedKeyword(_other.to_string())),
            }
        }
        TokenType::Identifier => {
            let id = token.as_any().downcast_ref::<Identifier>().unwrap();
            if !ctx.class_names.contains(&id.value) {
                return Err(Error::UnknownType(id.value.clone()));
            }
            Ok(Box::new(id.to_owned()))
        }
        _other => Err(Error::UnexpectedToken(_other)),
    }
}

/// Compile return type of a subroutine
fn compile_return_type(ctx: &mut Context, token: &Box<dyn Token>) -> Result<Box<dyn Token>, Error> {
    match token.token() {
        TokenType::Keyword => {
            let word = token.as_any().downcast_ref::<Keyword>().unwrap();
            match word.value.as_str() {
                tokenizer::INT | tokenizer::CHAR | tokenizer::BOOL | tokenizer::VOID => {
                    Ok(Box::new(word.to_owned()))
                }
                _other => Err(Error::UnexpectedKeyword(_other.to_string())),
            }
        }
        TokenType::Identifier => {
            let id = token.as_any().downcast_ref::<Identifier>().unwrap();
            if !ctx.class_names.contains(&id.value) {
                return Err(Error::UnknownType(id.value.clone()));
            }
            Ok(Box::new(id.to_owned()))
        }
        _other => Err(Error::UnexpectedToken(_other)),
    }
}
fn compile_classvardec(
    ctx: &mut Context,
    target: &mut ClassVarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_type = compile_type(ctx, &tokens.list[current_idx])?;
    current_idx += 1;
    loop {
        let tk = &tokens.list[current_idx];
        match tk.token() {
            TokenType::Symbol => {
                let s = tk.as_any().downcast_ref::<Symbol>().unwrap();
                match s.value {
                    ',' => target.var_delimiter.push(s.to_owned()),
                    ';' => {
                        // We got end of node symbol so we store it and go next
                        target.end_symbol = s.to_owned();
                        break;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol(s.value));
                    }
                }
            }
            TokenType::Identifier => {
                let i = tk.as_any().downcast_ref::<Identifier>().unwrap();
                target.var_names.push(i.to_owned());
            }
            _other => {
                return Err(Error::UnexpectedToken(_other));
            }
        }
        current_idx += 1;
    }
    Ok(current_idx)
}

fn compile_identifier(token: &Box<dyn Token>) -> Result<&Identifier, Error> {
    if !matches!(token.token(), TokenType::Identifier) {
        return Err(Error::UnexpectedToken(token.token()));
    }
    let id = token.as_any().downcast_ref::<Identifier>().unwrap();
    Ok(id)
}

fn compile_symbol(token: &Box<dyn Token>) -> Result<&Symbol, Error> {
    if !matches!(token.token(), TokenType::Symbol) {
        return Err(Error::UnexpectedToken(token.token()));
    }
    let s = token.as_any().downcast_ref::<Symbol>().unwrap();
    Ok(s)
}

/// Check and ingest all tokens related to current class
fn compile_class(
    ctx: &mut Context,
    class: &mut Class,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    // Check tokens from the head to see if they are valid class tokens
    let mut current_idx = token_index;
    let name_token = &tokens.list[current_idx];
    let name = compile_identifier(name_token)?;
    ctx.class_names.push(name.value.clone()); // store name in type table
    class.name = name.clone();
    current_idx += 1;
    let open_brace = compile_symbol(&tokens.list[current_idx])?;
    if open_brace.value != '{' {
        return Err(Error::UnexpectedSymbol(open_brace.value));
    }
    class.begin_symbol = open_brace.to_owned();
    current_idx += 1;
    loop {
        // Check for classVarDec, subroutineDec, or close brace until the end
        let t = &tokens.list[current_idx];
        match t.token() {
            TokenType::Symbol => {
                let close_brace = compile_symbol(t);
                // We ignore any errors for now
                if close_brace.is_ok() {
                    let s = close_brace.unwrap();
                    if s.value == '}' {
                        class.end_symbol = s.clone();
                        // Once we reach close brace we exit
                        break;
                    }
                    // We ignore invalid symbols for now
                    // return Err(String::from("Expected '}'"));
                }
                current_idx += 1;
            }
            TokenType::Keyword => {
                // We should be looking for keywords indicating classVarDec or subroutineDec
                let keyword = t.as_any().downcast_ref::<Keyword>().unwrap();
                match keyword.value.as_str() {
                    tokenizer::STATIC | tokenizer::FIELD => {
                        let mut cvd = ClassVarDec::new(keyword.clone());
                        current_idx = compile_classvardec(ctx, &mut cvd, tokens, current_idx + 1)?;
                        class.add_child(Box::new(cvd))?;
                    }
                    tokenizer::CONSTRUCTOR | tokenizer::FUNCTION | tokenizer::METHOD => {
                        let mut sd = SubroutineDec::new(keyword.clone());
                        current_idx = compile_subroutinedec(ctx, &mut sd, tokens, current_idx + 1)?;
                        class.add_child(Box::new(sd))?;
                    }
                    _ => {
                        // return Err(format!("Got unexpected keyword {}", keyword.value));
                        current_idx += 1;
                    }
                }
            }
            _other => {
                // return Err(String::from("Expected symbol or keyword"));
                current_idx += 1;
            }
        }
    }
    Ok(current_idx)
}

/// Parse specified file and generate an internal tree representation
pub fn parse_file(
    file_reader: &mut std::io::BufReader<std::fs::File>,
) -> Result<Box<dyn Node>, Error> {
    let tokens = generate_token_list(file_reader);
    let mut ctx = Context::new();
    let mut current_index = 0;
    let mut class = Class::new();
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
                        class.prefix = keyword.clone();
                        current_index =
                            compile_class(&mut ctx, &mut class, &tokens, current_index + 1)?;
                    }
                    _other => {
                        current_index += 1;
                    }
                }
            }
            TokenType::Symbol => {
                current_index += 1;
            }
            TokenType::Identifier => {
                current_index += 1;
            }
            TokenType::IntegerConst => {
                current_index += 1;
            }
            TokenType::StringConst => {
                current_index += 1;
            }
        }
    }
    Ok(Box::new(class))
}
