use std::ops::Deref;

use super::tokenizer;
use super::tokenizer::{
    generate_token_list, Identifier, Keyword, KeywordType, Symbol, Token, TokenList, TokenType,
    INDENT_STR, NEW_LINE,
};

const CLASS_VAR_DEC: &'static str = "classVarDec";
const SUBROUTINE_DEC: &'static str = "subroutineDec";
type ParseError = String;
type SerializeError = String;

pub trait Node {
    /// Serialize node at the specified indent level
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError>;
    /// Add child node.
    /// If node cannot add a child an error is returned
    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), ParseError>;
}

struct Root {
    children: Vec<Box<dyn Node>>,
}

impl Node for Root {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        for n in &self.children {
            // Root class is the root so all children are at level 0 indent
            n.serialize(output, 0)?;
        }
        Ok(())
    }

    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), ParseError> {
        Ok(self.children.push(node))
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
    fn new(prefix: Keyword) -> Class {
        Class {
            prefix: prefix,
            name: Identifier::new(),
            begin_symbol: Symbol::new(),
            end_symbol: Symbol::new(),
            children: Vec::new(),
        }
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

    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), ParseError> {
        Ok(self.children.push(node))
    }
}

struct ClassVarDec {
    prefix: Keyword,
    var_type: Identifier,
    var_names: Vec<Identifier>,
    var_delimiter: Vec<Symbol>,
    end_symbol: Symbol,
}

impl ClassVarDec {
    fn new(prefix: Keyword) -> ClassVarDec {
        ClassVarDec {
            prefix: prefix,
            var_type: Identifier::new(),
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
    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), ParseError> {
        Err(String::from("Cannot add children directly"))
    }
}

struct SubroutineDec {
    prefix: Keyword,
    children: Vec<Box<dyn Node>>,
}

impl SubroutineDec {
    fn new(prefix: Keyword) -> SubroutineDec {
        SubroutineDec {
            prefix: prefix,
            children: Vec::new(),
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
        // for c in &self.children {
        //     c.serialize(output, next_level);
        // }
        output.push_str(&end_tag);
        Ok(())
    }
    fn add_child(&mut self, node: Box<dyn Node>) -> Result<(), ParseError> {
        Ok(self.children.push(node))
    }
}

fn compile_classvardec(
    target: &mut ClassVarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, ParseError> {
    let mut current_idx = token_index;
    Ok(current_idx + 1)
}

fn compile_subroutinedec(
    target: &mut SubroutineDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, ParseError> {
    let mut current_idx = token_index;
    Ok(current_idx + 1)
}

fn compile_identifier(token: &Box<dyn Token>) -> Result<&Identifier, ParseError> {
    if !matches!(token.token(), TokenType::Identifier) {
        return Err(String::from("Expected Identifier token"));
    }
    let id = token.as_any().downcast_ref::<Identifier>().unwrap();
    Ok(id)
}

fn compile_symbol(token: &Box<dyn Token>, expected: char) -> Result<&Symbol, ParseError> {
    if !matches!(token.token(), TokenType::Symbol) {
        return Err(String::from("Expected Symbol token"));
    }
    let s = token.as_any().downcast_ref::<Symbol>().unwrap();
    if !(s.value == expected) {
        return Err(format!("Expected {}", expected));
    }
    Ok(s)
}

/// Check and ingest all tokens related to current class
fn compile_class(
    class: &mut Class,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, ParseError> {
    // Check tokens from the head to see if they are valid class tokens
    let mut current_idx = token_index;
    let name_token = &tokens.list[current_idx];
    let name = compile_identifier(name_token)?;
    // TODO: store name in type table
    class.name = name.clone();
    current_idx += 1;
    let open_brace = compile_symbol(&tokens.list[current_idx], '{')?;
    class.begin_symbol = open_brace.clone();
    current_idx += 1;
    loop {
        // Check for classVarDec, subroutineDec, or close brace until the end
        let t = &tokens.list[current_idx];
        match t.token() {
            TokenType::Symbol => {
                let close_brace = compile_symbol(t, '}');
                // We ignore any errors for now
                if close_brace.is_ok() {
                    class.end_symbol = close_brace.unwrap().clone();
                    // Once we reach close brace we exit
                    break;
                }
                current_idx += 1;
            }
            TokenType::Keyword => {
                // We should be looking for keywords indicating classVarDec or subroutineDec
                let keyword = t.as_any().downcast_ref::<Keyword>().unwrap();
                match keyword.value.as_str() {
                    tokenizer::STATIC | tokenizer::FIELD => {
                        let mut cvd = ClassVarDec::new(keyword.clone());
                        current_idx = compile_classvardec(&mut cvd, tokens, current_idx)?;
                        class.add_child(Box::new(cvd))?;
                    }
                    tokenizer::CONSTRUCTOR | tokenizer::FUNCTION | tokenizer::METHOD => {
                        let mut sd = SubroutineDec::new(keyword.clone());
                        current_idx = compile_subroutinedec(&mut sd, tokens, current_idx)?;
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

/// Convert token list to a compiled tree
fn parse_token_list(
    parent: &mut dyn Node,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, ParseError> {
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
                        let mut class = Class::new(keyword.clone());
                        current_index = compile_class(&mut class, tokens, current_index + 1)?;
                        parent.add_child(Box::new(class))?;
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
    Ok(current_index)
}

/// Generate parsed tree from given file reader
pub fn generate_tree(
    file_reader: &mut std::io::BufReader<std::fs::File>,
) -> Result<Box<dyn Node>, String> {
    let tokens = generate_token_list(file_reader);
    let mut root = Root {
        children: Vec::new(),
    };
    parse_token_list(&mut root, &tokens, 0)?;
    Ok(Box::new(root))
}
