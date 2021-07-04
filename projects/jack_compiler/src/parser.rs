use super::tokenizer;
use super::tokenizer::{
    generate_token_list, Identifier, IntegerConstant, Keyword, KeywordType, SerializeError,
    StringConstant, Symbol, Token, TokenList, INDENT_STR, NEW_LINE,
};
use std::collections::HashMap;

const CLASS_VAR_DEC: &'static str = "classVarDec";
const SUBROUTINE_DEC: &'static str = "subroutineDec";
const SUBROUTINE_BODY: &'static str = "subroutineBody";
const PARAMETER_LIST: &'static str = "parameterList";
const VAR_DEC: &'static str = "varDec";
const STATEMENTS: &'static str = "statements";
const TERM: &'static str = "term";
const RETURN_STATEMENT: &'static str = "returnStatement";
const DO_STATEMENT: &'static str = "doStatement";
const LET_STATEMENT: &'static str = "letStatement";
const IF_STATEMENT: &'static str = "ifStatement";
const WHILE_STATEMENT: &'static str = "whileStatement";
const EXPRESSION_LIST: &'static str = "expressionList";
const EXPRESSION: &'static str = "expression";
const CALL: &'static str = "call";
const PUSH: &'static str = "push";
const POP: &'static str = "pop";
const CONSTANT: &'static str = "constant";
const NEG: &'static str = "neg";
const NOT: &'static str = "not";
const LABEL: &'static str = "label";
const IF_GOTO: &'static str = "if-goto";
const GOTO: &'static str = "goto";
const LOCAL: &'static str = "local";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{file} {line}:{column} Got unexpected token at {index}: {token:?}")]
    UnexpectedToken {
        token: Token,
        index: usize,
        file: &'static str,
        line: u32,
        column: u32,
    },
    #[error("Got unexpected keyword: {0:?}")]
    UnexpectedKeyword(KeywordType),
    #[error("Got unknown type: {0}")]
    UnknownType(String),
    #[error("{file} {line}:{column} Got unexpected symbol at {index}: {symbol}")]
    UnexpectedSymbol {
        symbol: char,
        index: usize,
        file: &'static str,
        line: u32,
        column: u32,
    },
    #[error(
        "Not all tokens were consumed: token length: {token_length} token index: {current_index}"
    )]
    TokenLeftover {
        token_length: usize,
        current_index: usize,
    },
    #[error("Unexpected State: {0}")]
    UnexpectedState(String),
}

#[derive(Debug)]
enum SymbolCategory {
    Var,
    Argument,
    Static,
    Field,
}

fn var_type_to_symbol_type(var_type: &Token) -> SymbolType {
    match var_type {
        Token::Identifier(id) => SymbolType::Class(id.value.clone()),
        Token::Keyword(k) => match k.keyword() {
            KeywordType::Boolean => SymbolType::Boolean,
            KeywordType::Int => SymbolType::Int,
            KeywordType::Char => SymbolType::Char,
            _other => {
                panic!("Unexpected keyword type: {:?}", _other);
            }
        },
        _other => {
            panic!("Unexpected token type: {:?}", _other);
        }
    }
}

#[derive(Debug, Clone)]
enum SymbolType {
    Int,
    Char,
    Boolean,
    Class(String),
}

#[derive(Debug)]
struct SymbolTableEntry {
    category: SymbolCategory,
    /// Type name for this symbol
    symbol_type: SymbolType,
    /// Index of which this symbol showed up
    index: usize,
}

impl SymbolTableEntry {
    fn new(category: SymbolCategory, symbol_type: SymbolType, index: usize) -> SymbolTableEntry {
        SymbolTableEntry {
            category: category,
            symbol_type: symbol_type,
            index: index,
        }
    }
}

#[derive(Debug)]
struct ClassSymbolTable {
    table: HashMap<String, SymbolTableEntry>,
    static_count: usize,
    field_count: usize,
}

impl ClassSymbolTable {
    fn new() -> ClassSymbolTable {
        ClassSymbolTable {
            table: HashMap::new(),
            static_count: 0,
            field_count: 0,
        }
    }

    /// Add an entry to the symbol table and count up symbol index
    fn add_entry(&mut self, name: String, category: SymbolCategory, symbol_type: SymbolType) {
        match category {
            SymbolCategory::Static => {
                let entry = SymbolTableEntry::new(category, symbol_type, self.static_count);
                self.table.insert(name, entry);
                self.static_count += 1;
            }
            SymbolCategory::Field => {
                let entry = SymbolTableEntry::new(category, symbol_type, self.field_count);
                self.table.insert(name, entry);
                self.field_count += 1;
            }
            _other => panic!("Unexpected category: {:?}", _other),
        };
    }
}

#[derive(Debug)]
struct MethodSymbolTable {
    table: HashMap<String, SymbolTableEntry>,
    argument_count: usize,
    var_count: usize,
}

impl MethodSymbolTable {
    fn new() -> MethodSymbolTable {
        MethodSymbolTable {
            table: HashMap::new(),
            argument_count: 0,
            var_count: 0,
        }
    }

    /// Add an entry to the symbol table and count up symbol index
    fn add_entry(&mut self, name: String, category: SymbolCategory, symbol_type: SymbolType) {
        match category {
            SymbolCategory::Argument => {
                let entry = SymbolTableEntry::new(category, symbol_type, self.argument_count);
                self.table.insert(name, entry);
                self.argument_count += 1;
            }
            SymbolCategory::Var => {
                let entry = SymbolTableEntry::new(category, symbol_type, self.var_count);
                self.table.insert(name, entry);
                self.var_count += 1;
            }
            _other => panic!("Unexpected category: {:?}", _other),
        };
    }
}

#[derive(Debug, Clone)]
enum ReturnType {
    Void,
    Int,
    Char,
    Boolean,
    Class(String),
}

#[derive(Debug)]
struct ReturnTypeTable {
    table: HashMap<String, ReturnType>,
}

impl ReturnTypeTable {
    fn new() -> ReturnTypeTable {
        ReturnTypeTable {
            table: HashMap::new(),
        }
    }
}

/// Fill information of OS functions
fn init_os_functions(table: &mut ReturnTypeTable) {
    let str = ReturnType::Class(String::from("String"));
    let arr = ReturnType::Class(String::from("Array"));
    let funcs = [
        ("Math.multiply", ReturnType::Int),
        ("Math.divide", ReturnType::Int),
        ("Math.min", ReturnType::Int),
        ("Math.max", ReturnType::Int),
        ("Math.sqrt", ReturnType::Int),
        ("String.new", str.clone()),
        ("String.dispose", ReturnType::Int),
        ("String.length", ReturnType::Int),
        ("String.charAt", ReturnType::Char),
        ("String.setCharAt", ReturnType::Void),
        ("String.appendChar", str.clone()),
        ("String.eraseLastChar", ReturnType::Void),
        ("String.intValue", ReturnType::Int),
        ("String.setInt", ReturnType::Void),
        ("String.backSpace", ReturnType::Char),
        ("String.doubleQuote", ReturnType::Char),
        ("String.newLine", ReturnType::Char),
        ("Array.new", arr.clone()),
        ("Array.dispose", ReturnType::Void),
        ("Output.moveCursor", ReturnType::Void),
        ("Output.printChar", ReturnType::Void),
        ("Output.printString", ReturnType::Void),
        ("Output.printInt", ReturnType::Void),
        ("Output.println", ReturnType::Void),
        ("Output.backSpace", ReturnType::Void),
        ("Screen.clearScreen", ReturnType::Void),
        ("Screen.setColor", ReturnType::Void),
        ("Screen.drawPixel", ReturnType::Void),
        ("Screen.drawLine", ReturnType::Void),
        ("Screen.drawRectangle", ReturnType::Void),
        ("Screen.drawCircle", ReturnType::Void),
        ("Keyboard.keyPressed", ReturnType::Char),
        ("Keyboard.readChar", ReturnType::Char),
        ("Keyboard.readLine", str.clone()),
        ("Keyboard.readInt", ReturnType::Int),
        ("Memory.peek", ReturnType::Int),
        ("Memory.poke", ReturnType::Void),
        ("Memory.alloc", arr.clone()),
        ("Memory.dealloc", ReturnType::Void),
        ("Sys.halt", ReturnType::Void),
        ("Sys.error", ReturnType::Void),
        ("Sys.wait", ReturnType::Void),
    ];
    for (f, r) in funcs {
        table.table.insert(f.to_string(), r);
    }
}

/// Information gathered while parsing source code
pub struct ParseInfo {
    class_symbol_table: ClassSymbolTable,
    symbol_table_per_method: HashMap<String, MethodSymbolTable>,
    return_type: ReturnTypeTable,
}

impl ParseInfo {
    pub fn new() -> ParseInfo {
        let mut rt = ReturnTypeTable::new();
        init_os_functions(&mut rt);
        ParseInfo {
            class_symbol_table: ClassSymbolTable::new(),
            symbol_table_per_method: HashMap::new(),
            return_type: rt,
        }
    }
}

/// Function scoped state.
/// This gets reset for each function
struct FunctionScopeState {
    /// Name of current subroutine
    subroutine_name: String,
    /// Number of times a while occured in a single compile.
    /// Used to create unique label name per call.
    while_counter: usize,
    // Number of times an if occured in a single compile
    /// Used to create unique label name per call.
    if_counter: usize,
}

impl FunctionScopeState {
    fn new(subroutine_name: String) -> FunctionScopeState {
        FunctionScopeState {
            subroutine_name: subroutine_name,
            while_counter: 0,
            if_counter: 0,
        }
    }
}

/// State information of current compile
struct CompileState {
    /// Name of current class,
    class_name: String,
    func_state: FunctionScopeState,
}

impl CompileState {
    fn new(class_name: String) -> CompileState {
        CompileState {
            class_name: class_name,
            func_state: FunctionScopeState::new(String::from("")),
        }
    }

    /// Get full method name with ClassName.SubroutineName
    fn full_method_name(&self) -> String {
        format!("{}.{}", self.class_name, self.func_state.subroutine_name)
    }
}

pub struct Class {
    prefix: Keyword,
    name: Identifier,
    begin_symbol: Symbol,
    end_symbol: Symbol,
    class_vars: Vec<ClassVarDec>,
    subroutines: Vec<SubroutineDec>,
}

impl Class {
    fn new() -> Class {
        Class {
            prefix: Keyword::new(),
            name: Identifier::new(),
            begin_symbol: Symbol::new(),
            end_symbol: Symbol::new(),
            class_vars: Vec::new(),
            subroutines: Vec::new(),
        }
    }

    /// Serialize to XML
    pub fn serialize(
        &self,
        output: &mut String,
        indent_level: usize,
    ) -> Result<(), SerializeError> {
        let label = tokenizer::CLASS;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.prefix.serialize(output, next_level)?;
        self.name.serialize(output, next_level)?;
        self.begin_symbol.serialize(output, next_level)?;
        for c in &self.class_vars {
            c.serialize(output, next_level)?;
        }
        for s in &self.subroutines {
            s.serialize(output, next_level)?;
        }
        self.end_symbol.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    /// Compile to VM text
    pub fn compile(&self, info: &ParseInfo) -> Result<String, Error> {
        let mut output = String::from("");
        let mut state = CompileState::new(self.name.value.clone());
        // Iterate all subroutines
        for s in &self.subroutines {
            s.compile(info, &mut output, &mut state)?;
        }
        Ok(output)
    }
}

struct ClassVarDec {
    prefix: Keyword,
    var_type: Token, // var_type maybe a Keyword or an Identifier
    var_names: Vec<Identifier>,
    var_delimiter: Vec<Symbol>,
    end_symbol: Symbol,
}

impl ClassVarDec {
    fn new(prefix: Keyword) -> ClassVarDec {
        ClassVarDec {
            prefix: prefix,
            var_type: Token::Keyword(Keyword::new()),
            var_names: Vec::new(),
            var_delimiter: Vec::new(),
            end_symbol: Symbol::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        // number of delimiters should be one less than number of vars
        let var_num = self.var_names.len();
        let delim_num = self.var_delimiter.len();
        if var_num == 0 {
            return Err(SerializeError::UnexpectedState(String::from(
                "Missing variable name",
            )));
        } else if delim_num != (var_num - 1) {
            return Err(SerializeError::UnexpectedState(format!(
                "Number of delimiter should be var_num-1. var_num: {} delim_num: {}",
                var_num, delim_num
            )));
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
        self.var_names[0].serialize(output, next_level)?;
        if var_num > 1 {
            // multiple variables
            for i in 1..var_num {
                self.var_delimiter[i - 1].serialize(output, next_level)?;
                self.var_names[i].serialize(output, next_level)?;
            }
        }
        self.end_symbol.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

struct SubroutineDec {
    prefix: Keyword,
    return_type: Token, // return_type is a Keyword or an Identifier
    name: Identifier,
    param_list: ParameterList,
    body: SubroutineBody,
}

impl SubroutineDec {
    fn new(prefix: Keyword) -> SubroutineDec {
        SubroutineDec {
            prefix: prefix,
            return_type: Token::Keyword(Keyword::new()),
            name: Identifier::new(),
            param_list: ParameterList::new(),
            body: SubroutineBody::new(),
        }
    }

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

    pub fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &mut CompileState,
    ) -> Result<(), Error> {
        // Get name and number of variables
        let func_line = format!(
            "function {0}.{1} {2}{3}",
            state.class_name,
            self.name.value,
            self.body.variable_sum(),
            NEW_LINE
        );
        output.push_str(&func_line);
        // Create new function state
        state.func_state = FunctionScopeState::new(self.name.value.clone());
        // set parameters
        // set variables
        for s in &self.body.statements.list {
            s.compile(info, output, state)?;
        }
        Ok(())
    }
}

struct ParameterList {
    block: Block,
    param_type: Vec<Token>, // param_type is a Keyword or an Identifier
    name: Vec<Identifier>,
    delimiter: Vec<Symbol>,
}

impl ParameterList {
    fn new() -> ParameterList {
        ParameterList {
            block: Block::new(),
            param_type: Vec::new(),
            name: Vec::new(),
            delimiter: Vec::new(),
        }
    }

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
        self.block.start.serialize(output, indent_level)?;
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
        self.block.end.serialize(output, indent_level)?;
        Ok(())
    }
}

fn parse_parameter_list(
    ctx: &mut ParseInfo,
    target: &mut ParameterList,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let s = tokens.list[current_idx].symbol().unwrap();
    if s.value != '(' {
        return Err(Error::UnexpectedSymbol {
            symbol: s.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.block.start = s.to_owned();
    current_idx += 1;
    // This flag becomes true when we found a type for a parameter.
    // We use this flag to differentiate an identifier as a class name or param name
    let mut got_param_type = false;
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Symbol(s) => {
                match s.value {
                    ')' => {
                        // We got end of param list symbol so we store it and go next
                        target.block.end = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    ',' => {
                        // We got param delimiter
                        target.delimiter.push(s.to_owned());
                        current_idx += 1;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol {
                            symbol: _other,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                }
            }
            Token::Keyword(_) => {
                // should be a builtin type
                target
                    .param_type
                    .push(parse_type(ctx, tk, current_idx)?.to_owned());
                got_param_type = true;
                current_idx += 1;
            }
            Token::Identifier(id) => {
                if got_param_type {
                    // should be name of param
                    target.name.push(id.to_owned());
                    got_param_type = false
                } else {
                    // should be a class name
                    target
                        .param_type
                        .push(parse_type(ctx, tk, current_idx)?.to_owned());
                    got_param_type = true;
                }
                current_idx += 1;
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
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
    block: Block,
    variables: Vec<VarDec>,
    statements: StatementList,
}

impl SubroutineBody {
    fn new() -> SubroutineBody {
        SubroutineBody {
            block: Block::new(),
            variables: Vec::new(),
            statements: StatementList::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = SUBROUTINE_BODY;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.block.start.serialize(output, next_level)?;
        for v in &self.variables {
            if v.has_content() {
                v.serialize(output, next_level)?;
            }
        }
        self.statements.serialize(output, next_level)?;
        self.block.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    /// Get number of variables delcared within subroutine
    fn variable_sum(&self) -> usize {
        self.variables.iter().fold(0, |sum, v| sum + v.names.len())
    }
}

fn parse_subroutine_body(
    ctx: &mut ParseInfo,
    table: &mut MethodSymbolTable,
    target: &mut SubroutineBody,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let s = tokens.list[current_idx].symbol().unwrap();
    if s.value != '{' {
        return Err(Error::UnexpectedSymbol {
            symbol: s.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.block.start = s.to_owned();
    current_idx += 1;
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Symbol(s) => {
                match s.value {
                    '}' => {
                        // We got end of subroutine body symbol so we store it and go next
                        target.block.end = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol {
                            symbol: _other,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                }
            }
            Token::Keyword(k) => {
                match k.keyword() {
                    KeywordType::Var => {
                        // If we get 'var' it means we have a varDec
                        let mut vd = VarDec::new();
                        vd.prefix = k.to_owned();
                        current_idx = parse_var_dec(ctx, &mut vd, tokens, current_idx + 1)?;
                        // Add all declared vars to symbol table
                        for v in &vd.names {
                            table.add_entry(
                                v.string(),
                                SymbolCategory::Var,
                                var_type_to_symbol_type(&vd.var_type),
                            );
                        }
                        target.variables.push(vd);
                    }
                    KeywordType::Let
                    | KeywordType::If
                    | KeywordType::While
                    | KeywordType::Do
                    | KeywordType::Return => {
                        // If we get these keywords we have a statement
                        // We stay on same index (no increment) to read again from the statement keyword.
                        current_idx =
                            parse_statements(ctx, &mut target.statements, tokens, current_idx)?
                    }
                    _other => {
                        return Err(Error::UnexpectedKeyword(_other));
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
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
    prefix: Keyword,        // Should be 'var'
    var_type: Token,        // Should be a Keyword or an Identifier
    names: Vec<Identifier>, // List of names of variables
    delimiter: Vec<Symbol>, // Delimiters between variable names
    end: Symbol,
}

impl VarDec {
    fn new() -> VarDec {
        VarDec {
            prefix: Keyword::new(),
            var_type: Token::Keyword(Keyword::new()),
            names: Vec::new(),
            delimiter: Vec::new(),
            end: Symbol::new(),
        }
    }

    /// Returns true if there is any content to serialize
    fn has_content(&self) -> bool {
        self.names.len() > 0
    }

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

fn parse_var_dec(
    ctx: &mut ParseInfo,
    target: &mut VarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_type = parse_type(ctx, &tokens.list[current_idx], current_idx)?.to_owned();
    current_idx += 1;
    target
        .names
        .push(tokens.list[current_idx].identifier().unwrap().to_owned());
    current_idx += 1;
    // if next token is delimiter
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Symbol(s) => {
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
                            .push(tokens.list[current_idx].identifier().unwrap().to_owned());
                        current_idx += 1;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol {
                            symbol: _other,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

#[derive(Debug)]
struct Expression {
    terms: Vec<Term>,
    ops: Vec<Op>,
}

impl Expression {
    fn new() -> Expression {
        Expression {
            terms: Vec::new(),
            ops: Vec::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let term_len = self.terms.len();
        let op_len = self.ops.len();
        if term_len == 0 {
            return Err(SerializeError::UnexpectedState(String::from(
                "Expression must have one or more terms",
            )));
        } else if op_len != (term_len - 1) {
            return Err(SerializeError::UnexpectedState(format!(
                "Length of ops should be one less than length of terms: terms: {} ops: {}",
                term_len, op_len
            )));
        }
        let label = EXPRESSION;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.terms[0].serialize(output, next_level)?;
        for i in 1..term_len {
            self.ops[i - 1].serialize(output, next_level)?;
            self.terms[i].serialize(output, next_level)?;
        }
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        let term_len = self.terms.len();
        assert!(term_len > 0);
        assert_eq!(term_len - 1, self.ops.len());
        // compile via postfix approach
        self.terms[0].compile(info, output, state)?;
        for i in 1..term_len {
            self.terms[i].compile(info, output, state)?;
            self.ops[i - 1].compile(output)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Term {
    Integer(IntegerTerm),
    String(StringTerm),
    Keyword(KeywordTerm),
    VarName(VarNameTerm),
    ArrayVar(ArrayVarTerm),
    Subroutine(SubroutineCallTerm),
    ExpresssionInParenthesis(ExpressionInParenthesisTerm),
    UnaryOp(UnaryOpTerm),
}

impl Term {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        match self {
            Term::Integer(i) => i.serialize(output, indent_level),
            Term::String(s) => s.serialize(output, indent_level),
            Term::Keyword(k) => k.serialize(output, indent_level),
            Term::VarName(v) => v.serialize(output, indent_level),
            Term::ArrayVar(av) => av.serialize(output, indent_level),
            Term::Subroutine(sr) => sr.serialize(output, indent_level),
            Term::ExpresssionInParenthesis(e) => e.serialize(output, indent_level),
            Term::UnaryOp(u) => u.serialize(output, indent_level),
        }
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        match self {
            Term::Integer(i) => i.compile(info, output),
            Term::String(s) => s.compile(info, output),
            Term::ExpresssionInParenthesis(e) => e.expression.compile(info, output, state),
            Term::UnaryOp(u) => u.compile(info, output, state),
            Term::Subroutine(sr) => sr.compile(info, output, state),
            Term::VarName(v) => v.compile(info, output, state),
            Term::Keyword(k) => k.compile(info, output),
            _other => {
                println!("{}", output);
                panic!("NotImplemented");
            } // Term::ArrayVar(av) => av.serialize(output, indent_level),
        }
    }
}

#[derive(Debug)]
struct IntegerTerm {
    integer: IntegerConstant,
}

#[derive(Debug)]
struct StringTerm {
    string: StringConstant,
}

#[derive(Debug)]
struct KeywordTerm {
    keyword: Keyword,
}

#[derive(Debug)]
struct VarNameTerm {
    name: Identifier,
}
#[derive(Debug)]
struct ExpressionInParenthesisTerm {
    expression: Expression,
    block: Block,
}

#[derive(Debug)]
struct ArrayVarTerm {
    name: Identifier,
    arr: ArrayExpression,
}

#[derive(Debug)]
struct UnaryOpTerm {
    op: Symbol,
    term: Box<Term>,
}

#[derive(Debug)]
struct SubroutineCallTerm {
    call: SubroutineCall,
}

impl ArrayVarTerm {
    fn new() -> ArrayVarTerm {
        ArrayVarTerm {
            name: Identifier::new(),
            arr: ArrayExpression::new(),
        }
    }
}

impl IntegerTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.integer.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(&self, _context: &ParseInfo, output: &mut String) -> Result<(), Error> {
        let line = format!("{} {} {}{}", PUSH, CONSTANT, self.integer.value, NEW_LINE);
        output.push_str(&line);
        Ok(())
    }
}

impl StringTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.string.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(&self, _context: &ParseInfo, output: &mut String) -> Result<(), Error> {
        Ok(())
    }
}

impl VarNameTerm {
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

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        let method_table = info.symbol_table_per_method.get(&state.full_method_name());
        if method_table.is_some() {
            let entry = method_table.unwrap().table.get(&self.name.value).unwrap();
            match &entry.symbol_type {
                SymbolType::Class(_c) => {
                    panic!("NotImplemented");
                }
                _other => match &entry.category {
                    SymbolCategory::Argument => {
                        output.push_str(&format!("{} argument {}{}", PUSH, entry.index, NEW_LINE));
                        Ok(())
                    }
                    SymbolCategory::Var => {
                        output.push_str(&format!("{} {} {}{}", PUSH, LOCAL, entry.index, NEW_LINE));
                        Ok(())
                    }
                    _other => {
                        panic!("NotImplemented");
                    }
                },
            }
        } else {
            // look for class symbol table
            panic!("NotImplemented");
        }
    }
}

impl KeywordTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(&self, info: &ParseInfo, output: &mut String) -> Result<(), Error> {
        match self.keyword.value.as_str() {
            tokenizer::TRUE => {
                // true is -1 so we not a 0
                output.push_str(&format!(
                    "{0} {1} 0{nl}{2}{nl}",
                    PUSH,
                    CONSTANT,
                    NOT,
                    nl = NEW_LINE
                ));
                Ok(())
            }
            tokenizer::FALSE => {
                // false is 0
                output.push_str(&format!("{} {} 0{}", PUSH, CONSTANT, NEW_LINE));
                Ok(())
            }
            tokenizer::NULL => panic!("Not implemented"),
            tokenizer::THIS => panic!("Not implemented"),
            _other => panic!("Unexpected Keyword: {}", _other),
        }
    }
}

impl ExpressionInParenthesisTerm {
    fn new() -> ExpressionInParenthesisTerm {
        ExpressionInParenthesisTerm {
            expression: Expression::new(),
            block: Block::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.block.start.serialize(output, next_level)?;
        self.expression.serialize(output, next_level)?;
        self.block.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

impl ArrayVarTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.name.serialize(output, next_level)?;
        self.arr.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }
}

impl UnaryOpTerm {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.op.serialize(output, next_level)?;
        self.term.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        self.term.compile(info, output, state)?;
        match self.op.value {
            '-' => output.push_str(&format!("{}{}", NEG, NEW_LINE)),
            '~' => output.push_str(&format!("{}{}", NOT, NEW_LINE)),
            _other => panic!("Unexpected symbol: {}", _other),
        }
        Ok(())
    }
}

impl SubroutineCallTerm {
    fn new() -> SubroutineCallTerm {
        SubroutineCallTerm {
            call: SubroutineCall::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = TERM;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        self.call.serialize(output, indent_level + 1)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        self.call.call.compile(info, output, state)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Op {
    symbol: Symbol,
}

impl Op {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        self.symbol.serialize(output, indent_level)?;
        Ok(())
    }

    fn compile(&self, output: &mut String) -> Result<(), Error> {
        match self.symbol.value {
            '+' => output.push_str(&format!("add{}", NEW_LINE)),
            '-' => output.push_str(&format!("sub{}", NEW_LINE)),
            '=' => output.push_str(&format!("eq{}", NEW_LINE)),
            '>' => output.push_str(&format!("gt{}", NEW_LINE)),
            '<' => output.push_str(&format!("lt{}", NEW_LINE)),
            '&' => output.push_str(&format!("and{}", NEW_LINE)),
            '|' => output.push_str(&format!("or{}", NEW_LINE)),
            '~' => output.push_str(&format!("not{}", NEW_LINE)),
            '*' => output.push_str(&format!("{} Math.multiply 2{}", CALL, NEW_LINE)),
            '/' => output.push_str(&format!("{} Math.divide 2{}", CALL, NEW_LINE)),
            _other => panic!("Unexpected symbol: {}", _other),
        }
        Ok(())
    }
}

fn parse_term(
    ctx: &mut ParseInfo,
    tokens: &TokenList,
    token_index: usize,
) -> Result<(Term, usize), Error> {
    let mut current_idx = token_index;
    let t = &tokens.list[current_idx];
    match t {
        Token::IntegerConstant(ic) => {
            let i = IntegerTerm {
                integer: ic.to_owned(),
            };
            Ok((Term::Integer(i), current_idx + 1))
        }
        Token::StringConstant(sc) => {
            let s = StringTerm {
                string: sc.to_owned(),
            };
            Ok((Term::String(s), current_idx + 1))
        }
        Token::Keyword(kw) => {
            match kw.keyword() {
                KeywordType::This | KeywordType::Null | KeywordType::True | KeywordType::False => {
                    // KeywordConstant
                    let k = KeywordTerm {
                        keyword: kw.to_owned(),
                    };
                    Ok((Term::Keyword(k), current_idx + 1))
                }
                _other => Err(Error::UnexpectedKeyword(_other)),
            }
        }
        Token::Identifier(id) => {
            current_idx += 1;
            // Check next token to identify which term we have
            let next = &tokens.list[current_idx];
            match next {
                Token::Symbol(s) => {
                    match s.value {
                        '[' => {
                            // parse array
                            let mut arr = ArrayVarTerm::new();
                            arr.name = id.to_owned();
                            arr.arr.block.start = s.to_owned();
                            current_idx = parse_expression(
                                ctx,
                                &mut arr.arr.expression,
                                tokens,
                                current_idx + 1,
                            )?;
                            let close_brace = tokens.list[current_idx].symbol().unwrap();
                            if close_brace.value != ']' {
                                return Err(Error::UnexpectedSymbol {
                                    symbol: close_brace.value,
                                    index: current_idx,
                                    file: file!(),
                                    line: line!(),
                                    column: column!(),
                                });
                            }
                            arr.arr.block.end = close_brace.to_owned();
                            Ok((Term::ArrayVar(arr), current_idx + 1))
                        }
                        '(' => {
                            // parse subroutineCall (functionCall)
                            panic!("NotImplemented");
                        }
                        '.' => {
                            // parse subroutineCall (methodCall)
                            let mut mc = MethodCall::new();
                            mc.source_name = id.to_owned();
                            mc.dot = s.to_owned();
                            current_idx += 1;
                            let subroutine = tokens.list[current_idx].identifier().unwrap();
                            mc.method_name = subroutine.to_owned();
                            current_idx += 1;
                            let open_paren = tokens.list[current_idx].symbol().unwrap();
                            if open_paren.value != '(' {
                                return Err(Error::UnexpectedSymbol {
                                    symbol: open_paren.value,
                                    index: current_idx,
                                    file: file!(),
                                    line: line!(),
                                    column: column!(),
                                });
                            }
                            mc.parameter_block.start = open_paren.to_owned();
                            current_idx = parse_expression_list(
                                ctx,
                                &mut mc.parameters,
                                tokens,
                                current_idx + 1,
                            )?;
                            let close_paren = tokens.list[current_idx].symbol().unwrap();
                            if close_paren.value != ')' {
                                return Err(Error::UnexpectedSymbol {
                                    symbol: close_paren.value,
                                    index: current_idx,
                                    file: file!(),
                                    line: line!(),
                                    column: column!(),
                                });
                            }
                            mc.parameter_block.end = close_paren.to_owned();
                            let mut sc = SubroutineCallTerm::new();
                            sc.call.call = CallType::Method(mc);
                            Ok((Term::Subroutine(sc), current_idx + 1))
                        }
                        _other => {
                            // If we get any other symbol the first identifier is a varName
                            let t = VarNameTerm {
                                name: id.to_owned(),
                            };
                            Ok((Term::VarName(t), current_idx))
                        }
                    }
                }
                _other => {
                    // If we get any other token type the first identifier is a varName
                    let t = VarNameTerm {
                        name: id.to_owned(),
                    };
                    Ok((Term::VarName(t), current_idx))
                }
            }
        }
        Token::Symbol(s) => {
            match s.value {
                '(' => {
                    let mut exp = ExpressionInParenthesisTerm::new();
                    exp.block.start = s.to_owned();
                    current_idx =
                        parse_expression(ctx, &mut exp.expression, tokens, current_idx + 1)?;
                    let end = tokens.list[current_idx].symbol().unwrap();
                    if end.value != ')' {
                        return Err(Error::UnexpectedSymbol {
                            symbol: end.value,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                    exp.block.end = end.to_owned();
                    Ok((Term::ExpresssionInParenthesis(exp), current_idx + 1))
                }
                '-' | '~' => {
                    // Unary op + term
                    let (term, idx) = parse_term(ctx, tokens, current_idx + 1)?;
                    let uot = UnaryOpTerm {
                        op: s.to_owned(),
                        term: Box::new(term),
                    };
                    Ok((Term::UnaryOp(uot), idx))
                }
                _other => Err(Error::UnexpectedSymbol {
                    symbol: _other,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                }),
            }
        }
    }
}

fn parse_expression(
    ctx: &mut ParseInfo,
    target: &mut Expression,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    loop {
        let t = &tokens.list[current_idx];
        match t {
            Token::Symbol(s) => {
                match s.value {
                    '-' => {
                        // May be a unary op or a normal op
                        if target.terms.is_empty() {
                            // If no term appear before this we assume it is a unary op
                            let (term, idx) = parse_term(ctx, tokens, current_idx + 1)?;
                            let uot = UnaryOpTerm {
                                op: s.to_owned(),
                                term: Box::new(term),
                            };
                            target.terms.push(Term::UnaryOp(uot));
                            current_idx = idx;
                        } else {
                            // If we have another term before this we assume a normal op
                            let op = Op {
                                symbol: s.to_owned(),
                            };
                            target.ops.push(op);
                            current_idx += 1;
                        }
                    }
                    '~' => {
                        // Unary op + term
                        let (term, idx) = parse_term(ctx, tokens, current_idx + 1)?;
                        let uot = UnaryOpTerm {
                            op: s.to_owned(),
                            term: Box::new(term),
                        };
                        target.terms.push(Term::UnaryOp(uot));
                        current_idx = idx;
                    }
                    '+' | '*' | '/' | '&' | '|' | '<' | '>' | '=' => {
                        let op = Op {
                            symbol: s.to_owned(),
                        };
                        target.ops.push(op);
                        current_idx += 1;
                    }
                    ')' | ']' | ';' | ',' => {
                        // We've arrived to the end of parenthesis, array expression, line, or delimieter between expressions
                        break;
                    }
                    _other => {
                        let (term, idx) = parse_term(ctx, tokens, current_idx)?;
                        target.terms.push(term);
                        current_idx = idx;
                    }
                }
            }
            _other => {
                let (term, idx) = parse_term(ctx, tokens, current_idx)?;
                target.terms.push(term);
                current_idx = idx;
            }
        }
    }

    Ok(current_idx)
}

/// Start and end symbol for various blocks
#[derive(Debug)]
struct Block {
    start: Symbol,
    end: Symbol,
}

impl Block {
    fn new() -> Block {
        Block {
            start: Symbol::new(),
            end: Symbol::new(),
        }
    }
}

#[derive(Debug)]
enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}

impl Statement {
    /// Serialize statement at the specified indent level
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        match self {
            Statement::Let(l) => l.serialize(output, indent_level),
            Statement::If(i) => i.serialize(output, indent_level),
            Statement::While(w) => w.serialize(output, indent_level),
            Statement::Do(d) => d.serialize(output, indent_level),
            Statement::Return(r) => r.serialize(output, indent_level),
        }
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &mut CompileState,
    ) -> Result<(), Error> {
        match self {
            Statement::Let(l) => l.compile(info, output, state),
            Statement::If(i) => i.compile(info, output, state),
            Statement::While(w) => w.compile(info, output, state),
            Statement::Do(d) => d.compile(info, output, state),
            Statement::Return(r) => {
                let return_type = &info.return_type.table[&state.full_method_name()];
                r.compile(info, output, return_type)
            }
        }
    }
}

#[derive(Debug)]
struct ArrayExpression {
    block: Block,
    expression: Expression,
}

impl ArrayExpression {
    fn new() -> ArrayExpression {
        ArrayExpression {
            block: Block::new(),
            expression: Expression::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        self.block.start.serialize(output, indent_level)?;
        self.expression.serialize(output, indent_level)?;
        self.block.end.serialize(output, indent_level)?;
        Ok(())
    }
}

#[derive(Debug)]
struct LetStatement {
    keyword: Keyword,
    var_name: Identifier,
    array: Option<ArrayExpression>,
    assign: Symbol,
    right_hand_side: Expression,
    end: Symbol,
}

impl LetStatement {
    fn new() -> LetStatement {
        LetStatement {
            keyword: Keyword::new(),
            var_name: Identifier::new(),
            array: None,
            assign: Symbol::new(),
            right_hand_side: Expression::new(),
            end: Symbol::new(),
        }
    }
}

impl LetStatement {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = LET_STATEMENT;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        self.var_name.serialize(output, next_level)?;
        if self.array.is_some() {
            self.array.as_ref().unwrap().serialize(output, next_level)?;
        }
        self.assign.serialize(output, next_level)?;
        self.right_hand_side.serialize(output, next_level)?;
        self.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        if self.array.is_some() {
            println!("{}", output);
            // compile as array expression
            panic!("NotImplemented");
        } else {
            // compile as normal var
            self.right_hand_side.compile(info, output, state)?;
            // We should have the right hand value at top of stack so we assign that to var
            let method_table = info.symbol_table_per_method.get(&state.full_method_name());
            if method_table.is_some() {
                let entry = method_table
                    .unwrap()
                    .table
                    .get(&self.var_name.value)
                    .unwrap();
                match &entry.symbol_type {
                    SymbolType::Class(_c) => {
                        panic!("NotImplemented");
                    }
                    _other => {
                        // all other type can be assigned in a single line
                        output.push_str(&format!("{} {} {}{}", POP, LOCAL, entry.index, NEW_LINE));
                        Ok(())
                    }
                }
            } else {
                // look for class symbol table
                panic!("NotImplemented");
            }
        }
    }
}

/// 'else' block for an if statement.
/// This block may not exist
#[derive(Debug)]
struct ElseBlock {
    keyword: Keyword,
    statement_block: Block,
    statements: StatementList,
}

impl ElseBlock {
    fn new() -> ElseBlock {
        ElseBlock {
            keyword: Keyword::new(),
            statement_block: Block::new(),
            statements: StatementList::new(),
        }
    }
}

#[derive(Debug)]
struct IfStatement {
    keyword: Keyword,
    cond_block: Block,
    condition: Expression,
    statement_block: Block,
    statements: StatementList,
    else_block: Option<ElseBlock>,
}

impl IfStatement {
    fn new() -> IfStatement {
        IfStatement {
            keyword: Keyword::new(),
            cond_block: Block::new(),
            condition: Expression::new(),
            statement_block: Block::new(),
            statements: StatementList::new(),
            else_block: None,
        }
    }
}

impl IfStatement {
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = IF_STATEMENT;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        self.cond_block.start.serialize(output, next_level)?;
        self.condition.serialize(output, next_level)?;
        self.cond_block.end.serialize(output, next_level)?;
        self.statement_block.start.serialize(output, next_level)?;
        self.statements.serialize(output, next_level)?;
        self.statement_block.end.serialize(output, next_level)?;
        if self.else_block.is_some() {
            let eb = self.else_block.as_ref().unwrap();
            eb.keyword.serialize(output, next_level)?;
            eb.statement_block.start.serialize(output, next_level)?;
            eb.statements.serialize(output, next_level)?;
            eb.statement_block.end.serialize(output, next_level)?;
        }
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &mut CompileState,
    ) -> Result<(), Error> {
        let counter = state.func_state.if_counter;
        let cond_true_label = format!("IF_TRUE{}", counter);
        let cond_false_label = format!("IF_FALSE{}", counter);
        state.func_state.if_counter += 1;
        self.condition.compile(info, output, state)?;
        output.push_str(&format!(
            "{0}{nl}{1} {2}{nl}",
            NOT,
            IF_GOTO,
            cond_false_label,
            nl = NEW_LINE
        ));
        self.statements.compile(info, output, state)?;
        output.push_str(&format!("{} {}{}", GOTO, cond_true_label, NEW_LINE));
        output.push_str(&format!("{} {}{}", LABEL, cond_false_label, NEW_LINE));
        if self.else_block.is_some() {
            self.else_block
                .as_ref()
                .unwrap()
                .statements
                .compile(info, output, state)?;
        }
        output.push_str(&format!("{} {}{}", LABEL, cond_true_label, NEW_LINE));
        Ok(())
    }
}

#[derive(Debug)]
struct ExpressionList {
    list: Vec<Expression>,
    delimiter: Vec<Symbol>,
}

impl ExpressionList {
    fn new() -> ExpressionList {
        ExpressionList {
            list: Vec::new(),
            delimiter: Vec::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let list_len = self.list.len();
        let has_expression = list_len > 0;
        if has_expression {
            // delimiter is in between each param
            assert_eq!(list_len - 1, self.delimiter.len());
        } else {
            assert_eq!(0, self.delimiter.len());
        }
        let label = EXPRESSION_LIST;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        if has_expression {
            let next_level = indent_level + 1;
            self.list[0].serialize(output, next_level)?;
            for i in 1..list_len {
                self.delimiter[i - 1].serialize(output, next_level)?;
                self.list[i].serialize(output, next_level)?;
            }
        }
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        for e in &self.list {
            e.compile(info, output, state)?
        }
        Ok(())
    }
}

fn parse_expression_list(
    ctx: &mut ParseInfo,
    target: &mut ExpressionList,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Symbol(s) => {
                match s.value {
                    ')' => {
                        // End of expression list
                        break;
                    }
                    ',' => {
                        // We have another expression coming next
                        target.delimiter.push(s.to_owned());
                        current_idx += 1;
                    }
                    _other => {
                        // We have an expression so we parse it
                        let mut exp = Expression::new();
                        current_idx = parse_expression(ctx, &mut exp, tokens, current_idx)?;
                        target.list.push(exp);
                    }
                }
            }
            _other => {
                // We have an expression so we parse it
                let mut exp = Expression::new();
                current_idx = parse_expression(ctx, &mut exp, tokens, current_idx)?;
                target.list.push(exp);
            }
        }
    }
    Ok(current_idx)
}

#[derive(Debug)]
struct FunctionCall {
    name: Identifier,
    parameter_block: Block,
    parameters: ExpressionList,
}

impl FunctionCall {
    fn new() -> FunctionCall {
        FunctionCall {
            name: Identifier::new(),
            parameter_block: Block::new(),
            parameters: ExpressionList::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        self.name.serialize(output, indent_level)?;
        self.parameter_block.start.serialize(output, indent_level)?;
        self.parameters.serialize(output, indent_level)?;
        self.parameter_block.end.serialize(output, indent_level)?;
        Ok(())
    }
    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        self.parameters.compile(info, output, state)?;
        let line = format!(
            "{} {} {}{}",
            CALL,
            self.name.value,
            self.parameters.list.len(),
            NEW_LINE
        );
        output.push_str(&line);
        Ok(())
    }
}

#[derive(Debug)]
struct MethodCall {
    source_name: Identifier, // a className or varName
    dot: Symbol,
    method_name: Identifier,
    parameter_block: Block,
    parameters: ExpressionList,
}

impl MethodCall {
    fn new() -> MethodCall {
        MethodCall {
            source_name: Identifier::new(),
            dot: Symbol::new(),
            method_name: Identifier::new(),
            parameter_block: Block::new(),
            parameters: ExpressionList::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        self.source_name.serialize(output, indent_level)?;
        self.dot.serialize(output, indent_level)?;
        self.method_name.serialize(output, indent_level)?;
        self.parameter_block.start.serialize(output, indent_level)?;
        self.parameters.serialize(output, indent_level)?;
        self.parameter_block.end.serialize(output, indent_level)?;
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        self.parameters.compile(info, output, state)?;
        let caller = format!("{}.{}", self.source_name.value, self.method_name.value);
        let line = format!(
            "{} {} {}{}",
            CALL,
            caller,
            self.parameters.list.len(),
            NEW_LINE
        );
        output.push_str(&line);
        // if the method call's return type is void
        // we add an instruction to drop the implicit returned 0
        let rt = info.return_type.table.get(&caller).unwrap();
        if matches!(rt, ReturnType::Void) {
            output.push_str(&format!("pop temp 0{}", NEW_LINE));
        }

        Ok(())
    }
}

/// We use enum to restrict the child of SubroutineCall to be either FunctionCall or MethodCall
#[derive(Debug)]
enum CallType {
    Function(FunctionCall),
    Method(MethodCall),
}

impl CallType {
    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        match self {
            CallType::Function(f) => f.compile(info, output, state),
            CallType::Method(m) => m.compile(info, output, state),
        }
    }
}

#[derive(Debug)]
struct SubroutineCall {
    call: CallType,
}
impl SubroutineCall {
    fn new() -> SubroutineCall {
        SubroutineCall {
            call: CallType::Function(FunctionCall::new()),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        match &self.call {
            CallType::Function(func) => {
                func.serialize(output, indent_level)?;
            }
            CallType::Method(method) => {
                method.serialize(output, indent_level)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct DoStatement {
    keyword: Keyword,
    subroutine_call: SubroutineCall,
    end: Symbol,
}

impl DoStatement {
    fn new() -> DoStatement {
        DoStatement {
            keyword: Keyword::new(),
            subroutine_call: SubroutineCall::new(),
            end: Symbol::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = DO_STATEMENT;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        self.subroutine_call.serialize(output, next_level)?;
        self.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &CompileState,
    ) -> Result<(), Error> {
        self.subroutine_call.call.compile(info, output, state)?;
        Ok(())
    }
}
#[derive(Debug)]
struct StatementList {
    list: Vec<Statement>,
}

impl StatementList {
    fn new() -> StatementList {
        StatementList { list: Vec::new() }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = STATEMENTS;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        for s in &self.list {
            s.serialize(output, next_level)?;
        }
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &mut CompileState,
    ) -> Result<(), Error> {
        for s in &self.list {
            s.compile(info, output, state)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ReturnStatement {
    keyword: Keyword,
    expression: Option<Expression>,
    end: Symbol,
}

impl ReturnStatement {
    fn new() -> ReturnStatement {
        ReturnStatement {
            keyword: Keyword::new(),
            expression: None,
            end: Symbol::new(),
        }
    }
    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = RETURN_STATEMENT;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        if self.expression.is_some() {
            self.expression
                .as_ref()
                .unwrap()
                .serialize(output, next_level)?;
        }
        self.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        return_type: &ReturnType,
    ) -> Result<(), Error> {
        // Set return value based on return type
        match return_type {
            &ReturnType::Void => {
                // return 0 for void functions
                output.push_str(&format!("{} {} 0{}", PUSH, CONSTANT, NEW_LINE));
            }
            _other => {
                print!("{}", output);
                panic!("NotImplemented");
            }
        }
        output.push_str(&format!("return{}", NEW_LINE));
        Ok(())
    }
}

#[derive(Debug)]
struct WhileStatement {
    keyword: Keyword,
    condition: Block,
    expression: Expression,
    body: Block,
    statements: StatementList,
}

impl WhileStatement {
    fn new() -> WhileStatement {
        WhileStatement {
            keyword: Keyword::new(),
            condition: Block::new(),
            expression: Expression::new(),
            body: Block::new(),
            statements: StatementList::new(),
        }
    }

    fn serialize(&self, output: &mut String, indent_level: usize) -> Result<(), SerializeError> {
        let label = WHILE_STATEMENT;
        let indent = INDENT_STR.repeat(indent_level);
        let start_tag = format!("{0}<{1}>{2}", indent, label, NEW_LINE);
        let end_tag = format!("{0}</{1}>{2}", indent, label, NEW_LINE);
        output.push_str(&start_tag);
        let next_level = indent_level + 1;
        self.keyword.serialize(output, next_level)?;
        self.condition.start.serialize(output, next_level)?;
        self.expression.serialize(output, next_level)?;
        self.condition.end.serialize(output, next_level)?;
        self.body.start.serialize(output, next_level)?;
        self.statements.serialize(output, next_level)?;
        self.body.end.serialize(output, next_level)?;
        output.push_str(&end_tag);
        Ok(())
    }

    fn compile(
        &self,
        info: &ParseInfo,
        output: &mut String,
        state: &mut CompileState,
    ) -> Result<(), Error> {
        let counter = state.func_state.while_counter;
        let start_label = format!("WHILE_EXP{}", counter);
        let end_label = format!("WHILE_END{}", counter);
        state.func_state.while_counter += 1;
        // set start label
        output.push_str(&format!("{} {}{}", LABEL, start_label, NEW_LINE));
        // jump to end label if expression is false
        self.expression.compile(info, output, state)?;
        output.push_str(&format!(
            "{0}{nl}{1} {2}{nl}",
            NOT,
            IF_GOTO,
            end_label,
            nl = NEW_LINE
        ));
        // Run loop internal and jump back to start label.
        // Also place end label
        self.statements.compile(info, output, state)?;
        output.push_str(&format!(
            "{0} {1}{nl}{2} {3}{nl}",
            GOTO,
            start_label,
            LABEL,
            end_label,
            nl = NEW_LINE
        ));
        Ok(())
    }
}

fn parse_let_statement(
    ctx: &mut ParseInfo,
    target: &mut LetStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_name = tokens.list[current_idx].identifier().unwrap().to_owned();
    current_idx += 1;
    loop {
        let s = tokens.list[current_idx].symbol().unwrap();
        match s.value {
            ';' => {
                // Reached end of let statement
                target.end = s.to_owned();
                current_idx += 1;
                break;
            }
            '[' => {
                // got array expression
                let mut arr = ArrayExpression::new();
                arr.block.start = s.to_owned();
                current_idx = parse_expression(ctx, &mut arr.expression, tokens, current_idx + 1)?;
                let end_token = tokens.list[current_idx].symbol().unwrap();
                if end_token.value != ']' {
                    return Err(Error::UnexpectedSymbol {
                        symbol: end_token.value,
                        index: current_idx,
                        file: file!(),
                        line: line!(),
                        column: column!(),
                    });
                }
                arr.block.end = end_token.to_owned();
                target.array = Some(arr);
                current_idx += 1;
            }
            '=' => {
                // parse right hand side
                target.assign = s.to_owned();
                current_idx =
                    parse_expression(ctx, &mut target.right_hand_side, tokens, current_idx + 1)?;
            }
            _other => {
                return Err(Error::UnexpectedSymbol {
                    symbol: _other,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

fn parse_else_block(
    ctx: &mut ParseInfo,
    target: &mut ElseBlock,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let block_start = tokens.list[current_idx].symbol().unwrap();
    if block_start.value != '{' {
        return Err(Error::UnexpectedSymbol {
            symbol: block_start.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.statement_block.start = block_start.to_owned();
    current_idx = parse_statements(ctx, &mut target.statements, tokens, current_idx + 1)?;
    let block_end = tokens.list[current_idx].symbol().unwrap();
    if block_end.value != '}' {
        return Err(Error::UnexpectedSymbol {
            symbol: block_end.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.statement_block.end = block_end.to_owned();
    Ok(current_idx + 1)
}

fn parse_if_statement(
    ctx: &mut ParseInfo,
    target: &mut IfStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let cond_start = tokens.list[current_idx].symbol().unwrap();
    if cond_start.value != '(' {
        return Err(Error::UnexpectedSymbol {
            symbol: cond_start.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.cond_block.start = cond_start.to_owned();
    current_idx = parse_expression(ctx, &mut target.condition, tokens, current_idx + 1)?;
    let cond_end = tokens.list[current_idx].symbol().unwrap();
    if cond_end.value != ')' {
        return Err(Error::UnexpectedSymbol {
            symbol: cond_end.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.cond_block.end = cond_end.to_owned();
    current_idx += 1;
    let body_start = tokens.list[current_idx].symbol().unwrap();
    if body_start.value != '{' {
        return Err(Error::UnexpectedSymbol {
            symbol: body_start.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.statement_block.start = body_start.to_owned();
    current_idx = parse_statements(ctx, &mut target.statements, tokens, current_idx + 1)?;
    let body_end = tokens.list[current_idx].symbol().unwrap();
    if body_end.value != '}' {
        return Err(Error::UnexpectedSymbol {
            symbol: body_end.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.statement_block.end = body_end.to_owned();
    current_idx += 1;
    // Check if next token is 'else' and if so we parse the else block.
    // If it is anything else we assume it is some other statement and return
    let maybe_else = &tokens.list[current_idx];
    if !matches!(maybe_else, Token::Keyword(_)) {
        // Next token is not else so we return
        return Ok(current_idx);
    }
    let k = maybe_else.keyword().unwrap();
    if !matches!(k.keyword(), KeywordType::Else) {
        // Next keyword is not else so we return
        return Ok(current_idx);
    }
    // We got else so we parse else block
    let mut eb = ElseBlock::new();
    eb.keyword = k.to_owned();
    current_idx = parse_else_block(ctx, &mut eb, tokens, current_idx + 1)?;
    target.else_block = Some(eb);
    Ok(current_idx)
}

fn parse_subroutine_call(
    ctx: &mut ParseInfo,
    target: &mut SubroutineCall,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let source = tokens.list[current_idx].identifier().unwrap();
    current_idx += 1;
    // parsing branches depending on next symbol
    let next = tokens.list[current_idx].symbol().unwrap();
    match next.value {
        '(' => {
            // function call
            let mut f = FunctionCall::new();
            f.name = source.to_owned();
            f.parameter_block.start = next.to_owned();
            current_idx = parse_expression_list(ctx, &mut f.parameters, tokens, current_idx + 1)?;
            let end_token = tokens.list[current_idx].symbol().unwrap();
            if end_token.value != ')' {
                return Err(Error::UnexpectedSymbol {
                    symbol: end_token.value,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            f.parameter_block.end = end_token.to_owned();
            current_idx += 1;
            target.call = CallType::Function(f);
        }
        '.' => {
            // class/method call
            let mut m = MethodCall::new();
            m.source_name = source.to_owned();
            m.dot = next.to_owned();
            current_idx += 1;
            m.method_name = tokens.list[current_idx].identifier().unwrap().to_owned();
            current_idx += 1;
            let start = tokens.list[current_idx].symbol().unwrap();
            if start.value != '(' {
                return Err(Error::UnexpectedSymbol {
                    symbol: start.value,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            m.parameter_block.start = start.to_owned();
            current_idx = parse_expression_list(ctx, &mut m.parameters, tokens, current_idx + 1)?;
            let end = tokens.list[current_idx].symbol().unwrap();
            if end.value != ')' {
                return Err(Error::UnexpectedSymbol {
                    symbol: end.value,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            m.parameter_block.end = end.to_owned();
            current_idx += 1;
            target.call = CallType::Method(m);
        }
        _other => {
            return Err(Error::UnexpectedSymbol {
                symbol: _other,
                index: current_idx,
                file: file!(),
                line: line!(),
                column: column!(),
            });
        }
    }
    Ok(current_idx)
}

fn parse_do_statement(
    ctx: &mut ParseInfo,
    target: &mut DoStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let current_idx = parse_subroutine_call(ctx, &mut target.subroutine_call, tokens, token_index)?;
    let end_token = tokens.list[current_idx].symbol().unwrap();
    if end_token.value != ';' {
        return Err(Error::UnexpectedSymbol {
            symbol: end_token.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.end = end_token.to_owned();
    Ok(current_idx + 1)
}

fn parse_return_statement(
    ctx: &mut ParseInfo,
    target: &mut ReturnStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let tk = &tokens.list[current_idx];
    match tk {
        Token::Symbol(s) => {
            match s.value {
                ';' => {
                    // Reached end of statement
                    target.end = s.to_owned();
                    current_idx += 1;
                }
                _other => {
                    // Should be part of an expression
                    let mut e = Expression::new();
                    current_idx = parse_expression(ctx, &mut e, tokens, current_idx).unwrap();
                    target.expression = Some(e);
                    let end = tokens.list[current_idx].symbol().unwrap();
                    if end.value != ';' {
                        return Err(Error::UnexpectedSymbol {
                            symbol: end.value,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                    target.end = end.to_owned();
                    current_idx += 1;
                }
            }
        }
        _other => {
            // Should be part of an expression
            let mut e = Expression::new();
            current_idx = parse_expression(ctx, &mut e, tokens, current_idx).unwrap();
            target.expression = Some(e);
            let end = tokens.list[current_idx].symbol().unwrap();
            if end.value != ';' {
                return Err(Error::UnexpectedSymbol {
                    symbol: end.value,
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            target.end = end.to_owned();
            current_idx += 1;
        }
    }
    Ok(current_idx)
}

fn parse_while_statement(
    ctx: &mut ParseInfo,
    target: &mut WhileStatement,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let cond_start = tokens.list[current_idx].symbol().unwrap();
    if cond_start.value != '(' {
        return Err(Error::UnexpectedSymbol {
            symbol: cond_start.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.condition.start = cond_start.to_owned();
    current_idx = parse_expression(ctx, &mut target.expression, tokens, current_idx + 1)?;
    let cond_end = tokens.list[current_idx].symbol().unwrap();
    if cond_end.value != ')' {
        return Err(Error::UnexpectedSymbol {
            symbol: cond_end.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.condition.end = cond_end.to_owned();
    current_idx += 1;
    let body_start = tokens.list[current_idx].symbol().unwrap();
    if body_start.value != '{' {
        return Err(Error::UnexpectedSymbol {
            symbol: body_start.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.body.start = body_start.to_owned();
    current_idx = parse_statements(ctx, &mut target.statements, tokens, current_idx + 1)?;
    let body_end = tokens.list[current_idx].symbol().unwrap();
    if body_end.value != '}' {
        return Err(Error::UnexpectedSymbol {
            symbol: body_end.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    target.body.end = body_end.to_owned();
    Ok(current_idx + 1)
}

fn parse_statements(
    ctx: &mut ParseInfo,
    target: &mut StatementList,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Keyword(k) => match k.keyword() {
                KeywordType::Let => {
                    let mut l = LetStatement::new();
                    l.keyword = k.to_owned();
                    current_idx = parse_let_statement(ctx, &mut l, tokens, current_idx + 1)?;
                    target.list.push(Statement::Let(l));
                }
                KeywordType::If => {
                    let mut i = IfStatement::new();
                    i.keyword = k.to_owned();
                    current_idx = parse_if_statement(ctx, &mut i, tokens, current_idx + 1)?;
                    target.list.push(Statement::If(i));
                }
                KeywordType::While => {
                    let mut w = WhileStatement::new();
                    w.keyword = k.to_owned();
                    current_idx = parse_while_statement(ctx, &mut w, tokens, current_idx + 1)?;
                    target.list.push(Statement::While(w));
                }
                KeywordType::Do => {
                    let mut d = DoStatement::new();
                    d.keyword = k.to_owned();
                    current_idx = parse_do_statement(ctx, &mut d, tokens, current_idx + 1)?;
                    target.list.push(Statement::Do(d));
                }
                KeywordType::Return => {
                    let mut r = ReturnStatement::new();
                    r.keyword = k.to_owned();
                    current_idx = parse_return_statement(ctx, &mut r, tokens, current_idx + 1)?;
                    target.list.push(Statement::Return(r));
                }
                _other => {
                    return Err(Error::UnexpectedKeyword(_other));
                }
            },
            Token::Symbol(s) => {
                match s.value {
                    '}' => {
                        // Reached end of statements
                        // Since the end bracket belongs to parent node we don't increment index and just return
                        break;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol {
                            symbol: _other,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

fn keyword_to_return_type(keyword: KeywordType) -> ReturnType {
    match keyword {
        KeywordType::Void => ReturnType::Void,
        KeywordType::Boolean => ReturnType::Boolean,
        KeywordType::Int => ReturnType::Int,
        KeywordType::Char => ReturnType::Char,
        _other => panic!("Unexpected keyword type specified: {:?}", _other),
    }
}

fn token_to_return_type(t: &Token) -> ReturnType {
    match t {
        Token::Keyword(word) => match word.keyword() {
            KeywordType::Int | KeywordType::Char | KeywordType::Boolean | KeywordType::Void => {
                return keyword_to_return_type(word.keyword());
            }
            _other => panic!("Unexpected keyword: {:?}", _other),
        },
        Token::Identifier(id) => return ReturnType::Class(id.value.clone()),
        _other => panic!("Unexpected token: {:?}", _other),
    };
}

fn parse_subroutine_dec(
    info: &mut ParseInfo,
    target: &mut SubroutineDec,
    tokens: &TokenList,
    token_index: usize,
    class_name: &str,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    let mut symbol_table = MethodSymbolTable::new(); // Create new symbol table for every new subroutine
    let token = &tokens.list[current_idx];
    let rt = match token {
        Token::Keyword(word) => match word.keyword() {
            KeywordType::Int | KeywordType::Char | KeywordType::Boolean | KeywordType::Void => {
                token
            }
            _other => return Err(Error::UnexpectedKeyword(_other)),
        },
        Token::Identifier(_) => token,
        _other => {
            return Err(Error::UnexpectedToken {
                token: _other.to_owned(),
                index: current_idx,
                file: file!(),
                line: line!(),
                column: column!(),
            })
        }
    };
    target.return_type = rt.to_owned();
    current_idx += 1;
    target.name = tokens.list[current_idx].identifier().unwrap().to_owned();
    // Update return type
    let full_name = format!("{}.{}", class_name, target.name.string());
    info.return_type
        .table
        .insert(full_name.clone(), token_to_return_type(rt));
    current_idx = parse_parameter_list(info, &mut target.param_list, tokens, current_idx + 1)?;
    // add all parameters to symbol table
    for i in 0..target.param_list.name.len() {
        symbol_table.add_entry(
            target.param_list.name[i].string(),
            SymbolCategory::Argument,
            var_type_to_symbol_type(&target.param_list.param_type[i]),
        );
    }
    current_idx = parse_subroutine_body(
        info,
        &mut symbol_table,
        &mut target.body,
        tokens,
        current_idx,
    )?;
    // Add finished symbol table
    info.symbol_table_per_method.insert(full_name, symbol_table);
    Ok(current_idx)
}

fn parse_type<'a>(
    ctx: &mut ParseInfo,
    token: &'a Token,
    token_index: usize,
) -> Result<&'a Token, Error> {
    match token {
        Token::Keyword(word) => match word.keyword() {
            KeywordType::Int | KeywordType::Char | KeywordType::Boolean => Ok(token),
            _other => Err(Error::UnexpectedKeyword(_other)),
        },
        Token::Identifier(_id) => {
            // TODO:
            // We should check if a given class name is known, but since we don't have a concrete mechanism for that
            // (and also not required for a parser) we won't be doing it yet.
            // if !ctx.class_names.contains(&id.value) {
            //     return Err(Error::UnknownType(id.value.clone()));
            // }
            Ok(token)
        }
        _other => Err(Error::UnexpectedToken {
            token: _other.to_owned(),
            index: token_index,
            file: file!(),
            line: line!(),
            column: column!(),
        }),
    }
}

fn keyword_to_category(k: KeywordType) -> SymbolCategory {
    match k {
        KeywordType::Static => SymbolCategory::Static,
        KeywordType::Field => SymbolCategory::Field,
        _other => panic!("Unexpected keyword type specified: {:?}", _other),
    }
}

fn parse_class_var_dec(
    ctx: &mut ParseInfo,
    target: &mut ClassVarDec,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    let mut current_idx = token_index;
    target.var_type = parse_type(ctx, &tokens.list[current_idx], current_idx)?.to_owned();
    current_idx += 1;
    loop {
        let tk = &tokens.list[current_idx];
        match tk {
            Token::Symbol(s) => {
                match s.value {
                    ',' => target.var_delimiter.push(s.to_owned()),
                    ';' => {
                        // We got end of node symbol so we store it and go next
                        target.end_symbol = s.to_owned();
                        current_idx += 1;
                        break;
                    }
                    _other => {
                        return Err(Error::UnexpectedSymbol {
                            symbol: _other,
                            index: current_idx,
                            file: file!(),
                            line: line!(),
                            column: column!(),
                        });
                    }
                }
            }
            Token::Identifier(i) => {
                target.var_names.push(i.to_owned());
                ctx.class_symbol_table.add_entry(
                    i.string(),
                    keyword_to_category(target.prefix.keyword()),
                    var_type_to_symbol_type(&target.var_type),
                );
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
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

/// Check and ingest all tokens related to current class
fn parse_class(
    ctx: &mut ParseInfo,
    class: &mut Class,
    tokens: &TokenList,
    token_index: usize,
) -> Result<usize, Error> {
    // Check tokens from the head to see if they are valid class tokens
    let mut current_idx = token_index;
    let name = tokens.list[current_idx].identifier().unwrap();
    class.name = name.to_owned();
    current_idx += 1;
    let open_brace = tokens.list[current_idx].symbol().unwrap();
    if open_brace.value != '{' {
        return Err(Error::UnexpectedSymbol {
            symbol: open_brace.value,
            index: current_idx,
            file: file!(),
            line: line!(),
            column: column!(),
        });
    }
    class.begin_symbol = open_brace.to_owned();
    current_idx += 1;
    loop {
        // Check for classVarDec, subroutineDec, or close brace until the end
        let t = &tokens.list[current_idx];
        match t {
            Token::Symbol(close_brace) => {
                if close_brace.value != '}' {
                    return Err(Error::UnexpectedSymbol {
                        symbol: close_brace.value,
                        index: current_idx,
                        file: file!(),
                        line: line!(),
                        column: column!(),
                    });
                }
                class.end_symbol = close_brace.to_owned();
                // Once we reach close brace we exit
                break;
            }
            Token::Keyword(keyword) => {
                // We should be looking for keywords indicating classVarDec or subroutineDec
                match keyword.keyword() {
                    KeywordType::Static | KeywordType::Field => {
                        let mut cvd = ClassVarDec::new(keyword.to_owned());
                        current_idx = parse_class_var_dec(ctx, &mut cvd, tokens, current_idx + 1)?;
                        class.class_vars.push(cvd);
                    }
                    KeywordType::Constructor | KeywordType::Function | KeywordType::Method => {
                        let mut sd = SubroutineDec::new(keyword.to_owned());
                        current_idx = parse_subroutine_dec(
                            ctx,
                            &mut sd,
                            tokens,
                            current_idx + 1,
                            &class.name.value,
                        )?;
                        class.subroutines.push(sd);
                    }
                    _other => {
                        return Err(Error::UnexpectedKeyword(keyword.keyword()));
                    }
                }
            }
            _other => {
                return Err(Error::UnexpectedToken {
                    token: _other.to_owned(),
                    index: current_idx,
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
        }
    }
    Ok(current_idx)
}

/// Parse specified file and generate an internal tree representation
pub fn parse_file(
    info: &mut ParseInfo,
    file_reader: &mut std::io::BufReader<std::fs::File>,
) -> Result<Class, Error> {
    let tokens = generate_token_list(file_reader);
    let mut current_index = 0;
    let keyword = tokens.list[current_index].keyword().unwrap();
    if !matches!(keyword.keyword(), KeywordType::Class) {
        return Err(Error::UnexpectedKeyword(keyword.keyword()));
    }
    let mut class = Class::new();
    class.prefix = keyword.clone();
    current_index = parse_class(info, &mut class, &tokens, current_index + 1)?;
    if current_index != tokens.list.len() - 1 {
        // All tokens should be consumed
        return Err(Error::TokenLeftover {
            token_length: tokens.list.len(),
            current_index: current_index,
        });
    }
    Ok(class)
}
