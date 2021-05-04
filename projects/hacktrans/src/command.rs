type MemoryIndex = u32;
type CommandID = u32;

/// Type of arithmetic command
#[derive(Debug, Copy, Clone)]
pub enum ArithmeticType {
	Add,
	Sub,
	Neg,
	Eq,
	Gt,
	Lt,
	And,
	Or,
	Not,
}

/// Type of VM command
#[derive(Debug, Copy, Clone)]
pub enum CommandType {
	Arithmetic,
	Push,
	Pop,
	Label,
	GoTo,
	If,
	Function,
	Return,
	Call,
}

/// Type of segment for VM memory access (push, pop)
#[derive(Debug, Copy, Clone)]
pub enum SegmentType {
	Argument,
	Local,
	Static,
	Constant,
	This,
	That,
	Pointer,
	Temp,
}

#[derive(Debug)]
pub struct Arithmetic {
	command: CommandType,
	arithmetic: ArithmeticType,
	/// Unique ID of command within the same command group.
	/// This is used to create unique jump labels per command.
	/// If this is 0 (NULL_ID) it means it is not used for this command
	id: CommandID,
}

/// Counter for specific commands.
/// We need to count the number to create a unique ID to use as jump labels in each command.
/// Without this we will have clashing jump lables each time we use eq, gt, and lt.
pub struct Counter {
	pub eq: CommandID,
	pub gt: CommandID,
	pub lt: CommandID,
}

#[derive(Debug)]
pub struct MemoryAccess {
	command: CommandType,
	segment: SegmentType,
	index: MemoryIndex,
}

#[derive(Debug)]
pub struct ProgramFlow {
	command: CommandType,
	symbol: String,
}

#[derive(Debug)]
pub struct Function {
	command: CommandType,
	name: Option<String>,
	argument_num: Option<u16>,
}

/// Context of the current function calls.
/// Needed to generate function call/return labels
#[derive(Debug)]
pub struct Context {
	/// prefix is used as a unique string for marking labels unique to the input file
	pub prefix: String,
	/// Current Function caller.
	pub function_name: String,
	/// Number of functions called within given function
	pub function_count: u16,
}

pub const NULL_ID: CommandID = 0;

const ADD_ASM: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D+M
D=A+1
@SP
M=D
";

const SUB_ASM: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=M-D
D=A+1
@SP
M=D
";

const AND_ASM: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D&M
D=A+1
@SP
M=D
";

const OR_ASM: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D|M
D=A+1
@SP
M=D
";

const NEG_ASM: &'static str = "@SP
A=M
A=A-1
D=M
M=-M
D=A+1
@SP
M=D
";

const NOT_ASM: &'static str = "@SP
A=M
A=A-1
D=M
M=!M
D=A+1
@SP
M=D
";

/// General interface for all commands in VM
pub trait Command: std::fmt::Debug {
	/// Returns current command's command type
	fn command_type(&self) -> CommandType;
	fn to_asm_text(&self, context: &Context) -> Result<String, String>;
}

impl ProgramFlow {
	pub fn new(command: CommandType, symbol: String) -> ProgramFlow {
		ProgramFlow {
			command: command,
			symbol: symbol,
		}
	}
}

impl Command for ProgramFlow {
	fn command_type(&self) -> CommandType {
		self.command
	}
	fn to_asm_text(&self, context: &Context) -> Result<String, String> {
		let target_label = format!("{}.{}", context.prefix, self.symbol);
		match self.command {
			CommandType::Label => {
				let str = format!("({})\n", target_label);
				Ok(str)
			}
			CommandType::If => {
				// pop the top value of stack, and if it is not 0 we jump
				let str = format!(
					"@SP
AM=M-1
D=M
@{}
D;JNE
",
					target_label
				);
				Ok(str)
			}
			CommandType::GoTo => {
				// Jump to specified label
				let str = format!(
					"@{}
0;JMP
",
					target_label
				);
				Ok(str)
			}
			_other => Err(format!("Unsupported CommandType: {:?}", _other)),
		}
	}
}

impl Function {
	pub fn new(command: CommandType, name: Option<String>, arg_num: Option<u16>) -> Function {
		Function {
			command: command,
			name: name,
			argument_num: arg_num,
		}
	}
}

impl Command for Function {
	fn command_type(&self) -> CommandType {
		self.command
	}
	fn to_asm_text(&self, context: &Context) -> Result<String, String> {
		match self.command {
			CommandType::Function => {
				// set label and get ready for local variable initialization
				let mut str = format!(
					"({})
@SP
A=M
",
					self.name.as_ref().unwrap()
				);
				// initialize local variables to zero
				for _ in 0..self.argument_num.unwrap() {
					str.push_str(
						"M=0
A=A+1
",
					);
				}
				// Update stack pointer
				str.push_str(
					"D=A
@SP
M=D
",
				);
				Ok(str)
			}
			CommandType::Return => {
				let return_address = format!("{}.ret", context.prefix);
				// store return address,
				// push return value,
				// reposition stack pointer
				// restore segment address values
				// and jump to return address
				let str = format!(
					"@LCL
D=M
@5
A=D-A
D=M
@{0}
M=D
@SP
A=M-1
D=M
@ARG
A=M
M=D
// reposition stack pointer
D=A+1
@SP
M=D
// restore segment address
@LCL
A=M-1
D=M
@THAT
M=D
@LCL
A=M-1
A=A-1
D=M
@THIS
M=D
@LCL
D=M
@3
A=D-A
D=M
@ARG
M=D
@LCL
D=M
@4
A=D-A
D=M
@LCL
M=D
// goto return address
@{0}
A=M;JMP
",
					return_address
				);
				Ok(str)
			}
			CommandType::Call => {
				let return_label = format!("");
				// push return address
				// Save all register state (LCL, ARG, THIS, THAT)
				// Reposition ARG
				// Reposition SP
				// Goto Function label
				// Create return label
				let str = format!("",);
				Ok(str)
			}
			_other => Err(format!("Unsupported Function command: {:?}", _other)),
		}
	}
}

impl Command for MemoryAccess {
	fn command_type(&self) -> CommandType {
		self.command
	}
	fn to_asm_text(&self, context: &Context) -> Result<String, String> {
		let tmp_symbol = format!("{}.tmp", context.prefix);
		let static_symbol = format!("{}.{}", context.prefix, self.index);
		match self.command {
			CommandType::Push => match self.segment {
				SegmentType::Constant => {
					// push index value to global stack
					let str = format!(
						"@{}
D=A
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::Local => {
					// push value from local segment to global stack
					let str = format!(
						"@{}
D=A
@LCL
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::Argument => {
					// push value from argument segment to global stack
					let str = format!(
						"@{}
D=A
@ARG
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::This => {
					// push value from this segment to global stack
					let str = format!(
						"@{}
D=A
@THIS
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::That => {
					// push value from that segment to global stack
					let str = format!(
						"@{}
D=A
@THAT
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::Temp => {
					// push value from temp segment to global stack
					let str = format!(
						"@{}
D=A
@R5
A=D+A
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::Pointer => {
					// push value from pointer segment to global stack
					let str = format!(
						"@{}
D=A
@R3
A=D+A
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						self.index
					);
					Ok(str)
				}
				SegmentType::Static => {
					// push value from static segment to global stack
					let str = format!(
						"@{}
D=M
@SP
A=M
M=D
@SP
M=M+1
",
						static_symbol
					);
					Ok(str)
				}
			},
			CommandType::Pop => match self.segment {
				SegmentType::Local => {
					// move value from global stack to local segment
					let str = format!(
						"@{}
D=A
@LCL
D=D+M
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::Argument => {
					// move value from global stack to argument segment
					let str = format!(
						"@{}
D=A
@ARG
D=D+M
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::This => {
					// move value from global stack to this segment
					let str = format!(
						"@{}
D=A
@THIS
D=D+M
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::That => {
					// move value from global stack to that segment
					let str = format!(
						"@{}
D=A
@THAT
D=D+M
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::Temp => {
					// move value from global stack to temp segment (R5 to R12)
					let str = format!(
						"@{}
D=A
@R5
D=D+A
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::Pointer => {
					// move value from global stack to pointer segment (R3 to R4)
					let str = format!(
						"@{}
D=A
@R3
D=D+A
@{1}
M=D
@SP
AM=M-1
D=M
@{1}
A=M
M=D
",
						self.index, tmp_symbol
					);
					Ok(str)
				}
				SegmentType::Static => {
					// move value from global stack to static segment (variable)
					let str = format!(
						"@SP
AM=M-1
D=M
@{}
M=D
",
						static_symbol
					);
					Ok(str)
				}
				_other => Err(format!("Unsupported memory segment for Pop: {:?}", _other)),
			},
			_other => Err(format!("Unsupported MemoryAccessCommand: {:?}", _other)),
		}
	}
}

impl MemoryAccess {
	pub fn new(command: CommandType, segment: &str, index: &str) -> MemoryAccess {
		let seg = match segment {
			"argument" => SegmentType::Argument,
			"local" => SegmentType::Local,
			"static" => SegmentType::Static,
			"constant" => SegmentType::Constant,
			"this" => SegmentType::This,
			"that" => SegmentType::That,
			"temp" => SegmentType::Temp,
			"pointer" => SegmentType::Pointer,
			_other => panic!("Unknown segment specified: {:?}", _other),
		};
		let idx = str::parse::<MemoryIndex>(index);
		MemoryAccess {
			command: command,
			segment: seg,
			index: idx.unwrap(),
		}
	}
}

impl Arithmetic {
	pub fn new(arithmetic: ArithmeticType, id: CommandID) -> Arithmetic {
		Arithmetic {
			command: CommandType::Arithmetic,
			arithmetic: arithmetic,
			id: id,
		}
	}
}

impl Command for Arithmetic {
	fn command_type(&self) -> CommandType {
		self.command
	}

	fn to_asm_text(&self, _context: &Context) -> Result<String, String> {
		match self.arithmetic {
			ArithmeticType::Add => Ok(ADD_ASM.to_string()),
			ArithmeticType::Sub => Ok(SUB_ASM.to_string()),
			ArithmeticType::And => Ok(AND_ASM.to_string()),
			ArithmeticType::Or => Ok(OR_ASM.to_string()),
			ArithmeticType::Neg => Ok(NEG_ASM.to_string()),
			ArithmeticType::Not => Ok(NOT_ASM.to_string()),
			ArithmeticType::Eq => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsEq.{0}
D;JEQ
D=-1
(IsEq.{0})
@SP
A=M-1
A=A-1
M=!D
D=A+1
@SP
M=D
",
                self.id
            )),
			ArithmeticType::Lt => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsGe.{0}
D;JGE
D=-1
@WriteLtOutput.{0}
0;JMP
(IsGe.{0})
D=0
(WriteLtOutput.{0})
@SP
A=M-1
A=A-1
M=D
D=A+1
@SP
M=D
",
                self.id
            )),
			ArithmeticType::Gt => Ok(format!(
                // use the ID to create a unique jump label for each command
                "@SP
A=M
A=A-1
D=M
A=A-1
D=M-D
@IsGt.{0}
D;JGT
D=0
@WriteGtOutput.{0}
0;JMP
(IsGt.{0})
D=-1
(WriteGtOutput.{0})
@SP
A=M-1
A=A-1
M=D
D=A+1
@SP
M=D
",
                self.id
            )),
		}
	}
}
