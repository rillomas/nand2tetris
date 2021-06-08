use super::tokenizer;
use super::tokenizer::{
    generate_token_list, Identifier, IntegerConstant, Keyword, KeywordType, StringConstant, Symbol,
    Token, TokenList, TokenType, INDENT_STR, NEW_LINE,
};

const CLASS_VAR_DEC: &'static str = "classVarDec";
const SUBROUTINE_DEC: &'static str = "subroutineDec";
const SUBROUTINE_BODY: &'static str = "subroutineBody";
const PARAMETER_LIST: &'static str = "parameterList";
const VAR_DEC: &'static str = "varDec";
const STATEMENTS: &'static str = "statements";
const TERM: &'static str = "term";
type SerializeError = String;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{file} {line}:{column} Got unexpected token type: {token:?}")]
    UnexpectedToken {
        token: TokenType,
        file: &'static str,
        line: u32,
        column: u32,
    },
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
    return_type: Box<dyn Token>, // return_type is a Keyword or an Identifier
    name: Identifier,
    param_list: ParameterList,
    body: SubroutineBody,
}

impl SubroutineDec {
    fn new(prefix: Keyword) -> SubroutineDec {
        SubroutineDec {
            prefix: prefix,
            return_type: Box::new(Keyword::new()),
            name: Identifier::new(),
            param_list: ParameterList::new(),
            body: SubroutineBody::new(),
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
        self.param_list.serialize(output, next_level)?;
        self.body.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

struct ParameterList {
    start: Symbol,
    param_type: Vec<Box<dyn Token>>, // param_type is a Keyword or an Identifier
    name: Vec<Identifier>,
    delimiter: Vec<Symbol>,
    end: Symbol,
}

impl ParameterList {
    fn new() -> ParameterList {
        ParameterList {
            start: Symbol::new(),
            param_type: Vec::new(),
            name: Vec::new(),
            delimiter: Vec::new(),
            end: Symbol::new(),
        }
    }
}

impl Node for ParameterList {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let param_len = self.param_type.len();
        let has_param = param_len > 0;
        assert_eq!(param_len, self.name.len());
        if has_param {
            // delimiter is in between each param
            assert_eq!(self.param_type.len() - 1, self.delimiter.len());
        } else {
            assert_eq!(0, self.delimiter.len());
        }
        self.start.serialize(output, indent_level)?;
        let label = PARAMETER_LIST;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        let next_level = indent_level + 1;
        output.push_str(&start_tag);
        if has_param {
            self.param_type[0].serialize(output, next_level)?;
            self.name[0].serialize(output, next_level)?;
            for i in 1..param_len {
                // we have one less delimiter for each type/param name pair
                self.delimiter[i - 1].serialize(output, next_level)?;
                self.param_type[i].serialize(output, next_level)?;
                self.name[i].serialize(output, next_level)?;
            }
        }
        output.push_str(&end_tag);
        self.end.serialize(output, indent_level)?;
        Ok(())
    }
}
fn compile_parameter_list(
    ctx: &mut Context,
    target: &mut ParameterList,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let s = compile_symbol(&tokens.list[current_idx])?.to_owned();
    if s.value != '(' {
        return Err(Error::UnexpectedSymbol(s.value));
    }
    target.start = s;
    current_idx += 1;
    // This flag becomes true when we found a type for a parameter.
    // We use this flag to differentiate an identifier as a class name or param name
    let mut got_param_type = false;
    loop {
        let tk = &tokens.list[current_idx];
        match tk.token() {
            TokenType::Symbol => {
                let s = tk.as_any().downcast_ref::<Symbol>().unwrap();
                match s.value {
                    ')' => {
                        // We got end of param list symbol so we store it and go next
                        target.end = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    ',' => {
                        // We got param delimiter
                        target.delimiter.push(s.to_owned());
                        current_idx += 1;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol(_other));
                    }
                }
            }
            TokenType::Keyword => {
                // should be a builtin type
                target.param_type.push(compile_type(ctx, tk)?.boxed_clone());
                got_param_type = true;
                current_idx += 1;
            }
            TokenType::Identifier => {
                if got_param_type {
                    // should be name of param
                    target.name.push(compile_identifier(tk)?.to_owned());
                    got_param_type = false
                } else {
                    // should be a class name
                    target.param_type.push(compile_type(ctx, tk)?.boxed_clone());
                    got_param_type = true;
                }
                current_idx += 1;
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

struct SubroutineBody {
    start: Symbol,
    variables: VarDec,
    statements: StatementList,
    end: Symbol,
}

impl SubroutineBody {
    fn new() -> SubroutineBody {
        SubroutineBody {
            start: Symbol::new(),
            variables: VarDec::new(),
            statements: StatementList::new(),
            end: Symbol::new(),
        }
    }
}

impl Node for SubroutineBody {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = SUBROUTINE_BODY;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.start.serialize(output, next_level)?;
        if self.variables.has_content() {
            self.variables.serialize(output, next_level)?;
        }
        self.statements.serialize(output, next_level)?;
        self.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

fn compile_subroutine_body(
    ctx: &mut Context,
    target: &mut SubroutineBody,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let s = compile_symbol(&tokens.list[current_idx])?.to_owned();
    if s.value != '{' {
        return Err(Error::UnexpectedSymbol(s.value));
    }
    target.start = s;
    current_idx += 1;
    loop {
        let tk = &tokens.list[current_idx];
        match tk.token() {
            TokenType::Symbol => {
                let s = tk.as_any().downcast_ref::<Symbol>().unwrap();
                match s.value {
                    '}' => {
                        // We got end of subroutine body symbol so we store it and go next
                        target.end = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol(_other));
                    }
                }
            }
            TokenType::Keyword => {
                let k = tk.as_any().downcast_ref::<Keyword>().unwrap();
                match k.value.as_str() {
                    tokenizer::VAR => {
                        // If we get 'var' it means we have a varDec
                        let mut vd = VarDec::new();
                        vd.prefix = k.to_owned();
                        current_idx = compile_var_dec(ctx, &mut vd, tokens, current_idx + 1)?
                    }
                    tokenizer::LET
                    | tokenizer::IF
                    | tokenizer::WHILE
                    | tokenizer::DO
                    | tokenizer::RETURN => {
                        // If we get these tokenizers we have a statement
                        let mut s = StatementList::new();
                        // We stay on same index (no increment) to read again from the statement keyword.
                        current_idx = compile_statements(ctx, &mut s, tokens, current_idx)?
                    }
                    _other => {
                        return Err(Error::UnexpectedKeyword(_other.to_string()));
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

struct VarDec {
    prefix: Keyword,          // Should be 'var'
    var_type: Box<dyn Token>, // Should be a Keyword or an Identifier
    names: Vec<Identifier>,   // List of names of variables
    delimiter: Vec<Symbol>,   // Delimiters between variable names
    end: Symbol,
}

impl VarDec {
    fn new() -> VarDec {
        VarDec {
            prefix: Keyword::new(),
            var_type: Box::new(Keyword::new()),
            names: Vec::new(),
            delimiter: Vec::new(),
            end: Symbol::new(),
        }
    }

    /// Returns true if there is any content to serialize
    fn has_content(&self) -> bool {
        self.names.len() > 0
    }
}

impl Node for VarDec {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let var_num = self.names.len();
        assert!(var_num > 0);
        assert_eq!(self.delimiter.len(), var_num - 1);
        let label = VAR_DEC;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.prefix.serialize(output, next_level)?;
        self.var_type.serialize(output, next_level)?;
        self.names[0].serialize(output, next_level)?;
        for i in 1..var_num {
            self.delimiter[i - 1].serialize(output, next_level)?;
            self.names[i].serialize(output, next_level)?;
        }
        self.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

fn compile_var_dec(
    ctx: &mut Context,
    target: &mut VarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_type = compile_type(ctx, &tokens.list[current_idx])?.boxed_clone();
    current_idx += 1;
    target
        .names
        .push(compile_identifier(&tokens.list[current_idx])?.to_owned());
    current_idx += 1;
    // if next token is delimiter
    loop {
        let tk = &tokens.list[current_idx];
        match tk.token() {
            TokenType::Symbol => {
                let s = tk.as_any().downcast_ref::<Symbol>().unwrap();
                match s.value {
                    ';' => {
                        // We got end of VarDec symbol so we store it and go next
                        target.end = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    ',' => {
                        // We found a delimiter so we read another varName
                        target.delimiter.push(s.to_owned());
                        current_idx += 1;
                        target
                            .names
                            .push(compile_identifier(&tokens.list[current_idx])?.to_owned());
                        current_idx += 1;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol(_other));
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

struct Expression {
    terms: Vec<Box<dyn Term>>,
    ops: Vec<Op>,
}

impl Expression {
    fn new() -> Expression {
        Expression {
            terms: Vec::new(),
            ops: Vec::new(),
        }
    }
}
trait Term {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError>;
}

struct IntegerTerm {}

struct StringTerm {}

struct KeywordTerm {}

struct VarNameTerm {
    name: Identifier,
}

impl Term for VarNameTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.name.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

struct Op {
    symbol: Symbol,
}

fn compile_expression(
    ctx: &mut Context,
    target: &mut Expression,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    loop {
        let t = &tokens.list[current_idx];
        match t.token() {
            TokenType::Identifier => {
                let id = compile_identifier(t)?;
                current_idx += 1;
                // Check next token to identify which term we have
                let next = &tokens.list[current_idx];
                let mut is_var_name = false;
                match next.token() {
                    TokenType::Symbol => {
                        let s = compile_symbol(next)?;
                        match s.value {
                            '[' => {
                                // compile array
                                return Err(Error::UnexpectedSymbol(s.value));
                            }
                            '(' => {
                                // compile subroutineCall
                                return Err(Error::UnexpectedSymbol(s.value));
                            }
                            '.' => {
                                // compile subroutineCall
                                return Err(Error::UnexpectedSymbol(s.value));
                            }
                            _other => {
                                // If we get any other symbol the first identifier is a varName
                                is_var_name = true;
                            }
                        }
                    }
                    _other => {
                        // If we get any other token type the first identifier is a varName
                        is_var_name = true;
                    }
                }
                if is_var_name {
                    let t = VarNameTerm {
                        name: id.to_owned(),
                    };
                    target.terms.push(Box::new(t));
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }

    Ok(current_idx)
}

trait Statement {
    /// Serialize statement at the specified indent level
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError>;
}

struct LetStatement {
    prefix: Keyword,
    var_name: Identifier,
    arr_start: Option<Symbol>,
    arr_expression: Option<Expression>,
    arr_end: Option<Symbol>,
    assign: Symbol,
    right_hand_side: Expression,
    end: Symbol,
}

impl LetStatement {
    fn new() -> LetStatement {
        LetStatement {
            prefix: Keyword::new(),
            var_name: Identifier::new(),
            arr_start: None,
            arr_expression: None,
            arr_end: None,
            assign: Symbol::new(),
            right_hand_side: Expression::new(),
            end: Symbol::new(),
        }
    }
}

impl Statement for LetStatement {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        Ok(())
    }
}

struct StatementList {
    list: Vec<Box<dyn Statement>>,
}

impl StatementList {
    fn new() -> StatementList {
        StatementList { list: Vec::new() }
    }
}

impl Node for StatementList {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = STATEMENTS;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        // let next_level = indent_level + 1;
        output.push_str(&end_tag);
        Ok(())
    }
}

fn compile_let_statement(
    ctx: &mut Context,
    target: &mut LetStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_name = compile_identifier(&tokens.list[current_idx])?.to_owned();
    current_idx += 1;
    loop {
        let s = compile_symbol(&tokens.list[current_idx])?;
        match s.value {
            ';' => {
                // Reached end of let statement
                target.end = s.to_owned();
                current_idx += 1;
                break;
            }
            '[' => {
                target.arr_start = Some(s.to_owned());
                let mut exp = Expression::new();
                current_idx = compile_expression(ctx, &mut exp, tokens, current_idx + 1)?;
                target.arr_expression = Some(exp);
                target.arr_end = Some(compile_symbol(&tokens.list[current_idx])?.to_owned());
                current_idx += 1;
            }
            '=' => {
                // parse right hand side
                target.assign = s.to_owned();
                let mut exp = Expression::new();
                current_idx = compile_expression(ctx, &mut exp, tokens, current_idx + 1)?;
                target.right_hand_side = exp;
            }
            _other => {
                return Err(Error::UnexpectedSymbol(_other));
            }
        }
    }
    Ok(current_idx)
}

fn compile_statements(
    ctx: &mut Context,
    target: &mut StatementList,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    loop {
        let tk = &tokens.list[current_idx];
        match tk.token() {
            TokenType::Keyword => {
                let k = tk.as_any().downcast_ref::<Keyword>().unwrap();
                match k.value.as_str() {
                    tokenizer::LET => {
                        let mut l = LetStatement::new();
                        l.prefix = k.to_owned();
                        current_idx = compile_let_statement(ctx, &mut l, tokens, current_idx + 1)?;
                        target.list.push(Box::new(l));
                    }
                    tokenizer::IF => {
                        current_idx += 1;
                    }
                    tokenizer::WHILE => {
                        current_idx += 1;
                    }
                    tokenizer::DO => {
                        current_idx += 1;
                    }
                    tokenizer::RETURN => {
                        current_idx += 1;
                    }
                    _other => {
                        return Err(Error::UnexpectedKeyword(_other.to_string()));
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

fn compile_subroutine_dec(
    ctx: &mut Context,
    target: &mut SubroutineDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.return_type = compile_return_type(ctx, &tokens.list[current_idx])?.boxed_clone();
    current_idx += 1;
    target.name = compile_identifier(&tokens.list[current_idx])?.to_owned();
    current_idx = compile_parameter_list(ctx, &mut target.param_list, tokens, current_idx + 1)?;
    current_idx = compile_subroutine_body(ctx, &mut target.body, tokens, current_idx)?;
    Ok(current_idx)
}

fn compile_type<'a>(ctx: &mut Context, token: &'a Box<dyn Token>) -> Result<&'a dyn Token, Error> {
    match token.token() {
        TokenType::Keyword => {
            let word = token.as_any().downcast_ref::<Keyword>().unwrap();
            match word.value.as_str() {
                tokenizer::INT | tokenizer::CHAR | tokenizer::BOOL => Ok(word),
                _other => Err(Error::UnexpectedKeyword(_other.to_string())),
            }
        }
        TokenType::Identifier => {
            let id = token.as_any().downcast_ref::<Identifier>().unwrap();
            // TODO:
            // We should check if a given class name is known, but since we don't have a concrete mechanism for that
            // (and also not required for a parser) we won't be doing it yet.
            // if !ctx.class_names.contains(&id.value) {
            //     return Err(Error::UnknownType(id.value.clone()));
            // }
            Ok(id)
        }
        _other => Err(Error::UnexpectedToken {
            token: _other,
            file: file!(),
            line: line!(),
            column: column!(),
        }),
    }
}

/// Compile return type of a subroutine
fn compile_return_type<'a>(
    ctx: &mut Context,
    token: &'a Box<dyn Token>,
) -> Result<&'a dyn Token, Error> {
    match token.token() {
        TokenType::Keyword => {
            let word = token.as_any().downcast_ref::<Keyword>().unwrap();
            match word.value.as_str() {
                tokenizer::INT | tokenizer::CHAR | tokenizer::BOOL | tokenizer::VOID => Ok(word),
                _other => Err(Error::UnexpectedKeyword(_other.to_string())),
            }
        }
        TokenType::Identifier => {
            let id = token.as_any().downcast_ref::<Identifier>().unwrap();
            if !ctx.class_names.contains(&id.value) {
                return Err(Error::UnknownType(id.value.clone()));
            }
            Ok(id)
        }
        _other => Err(Error::UnexpectedToken {
            token: _other,
            file: file!(),
            line: line!(),
            column: column!(),
        }),
    }
}
fn compile_class_var_dec(
    ctx: &mut Context,
    target: &mut ClassVarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_type = compile_type(ctx, &tokens.list[current_idx])?.boxed_clone();
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
                return Err(Error::UnexpectedToken {
                    token: _other,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
        current_idx += 1;
    }
    Ok(current_idx)
}

fn compile_identifier(token: &Box<dyn Token>) -> Result<&Identifier, Error> {
    if !matches!(token.token(), TokenType::Identifier) {
        return Err(Error::UnexpectedToken {
            token: token.token(),
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    let id = token.as_any().downcast_ref::<Identifier>().unwrap();
    Ok(id)
}

fn compile_symbol(token: &Box<dyn Token>) -> Result<&Symbol, Error> {
    if !matches!(token.token(), TokenType::Symbol) {
        return Err(Error::UnexpectedToken {
            token: token.token(),
            file: file!(),
            line: line!(),
            column: column!(),
        });
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
                // We ignore any errors for now because we want to parse as much as possible
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
                        current_idx =
                            compile_class_var_dec(ctx, &mut cvd, tokens, current_idx + 1)?;
                        class.add_child(Box::new(cvd))?;
                    }
                    tokenizer::CONSTRUCTOR | tokenizer::FUNCTION | tokenizer::METHOD => {
                        let mut sd = SubroutineDec::new(keyword.clone());
                        current_idx =
                            compile_subroutine_dec(ctx, &mut sd, tokens, current_idx + 1)?;
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
