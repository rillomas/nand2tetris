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
	// Function,
	// Return,
	// Call,
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

pub const NULL_ID: CommandID = 0;

const ADD_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D+M
D=A+1
@SP
M=D
";

const SUB_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=M-D
D=A+1
@SP
M=D
";

const AND_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D&M
D=A+1
@SP
M=D
";

const OR_STR: &'static str = "@SP
A=M
A=A-1
D=M
A=A-1
M=D|M
D=A+1
@SP
M=D
";

const NEG_STR: &'static str = "@SP
A=M
A=A-1
D=M
M=-M
D=A+1
@SP
M=D
";

const NOT_STR: &'static str = "@SP
A=M
A=A-1
D=M
M=!M
D=A+1
@SP
M=D
";

pub trait Command: std::fmt::Debug {
	/// Returns current command's command type
	fn command_type(&self) -> CommandType;
	/// prefix is used as a unique string for marking labels unique to the input file
	fn to_asm_text(&self, prefix: &String) -> Result<String, String>;
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
	fn to_asm_text(&self, prefix: &String) -> Result<String, String> {
		let target_label = format!("{}.{}", prefix, self.symbol);
		match self.command {
			CommandType::Label => {
				let str = format!("({})\n", target_label);
				Ok(str.to_string())
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
				Ok(str.to_string())
			}
			CommandType::GoTo => {
				// Jump to specified label
				let str = format!(
					"@{}
0;JMP
",
					target_label
				);
				Ok(str.to_string())
			}
			_other => Err(format!("Unsupported CommandType: {:?}", _other)),
		}
	}
}

impl Command for MemoryAccess {
	fn command_type(&self) -> CommandType {
		self.command
	}
	fn to_asm_text(&self, prefix: &String) -> Result<String, String> {
		let tmp_symbol = format!("{}.tmp", prefix);
		let static_symbol = format!("{}.{}", prefix, self.index);
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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
					Ok(str.to_string())
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

	fn to_asm_text(&self, _prefix: &String) -> Result<String, String> {
		match self.arithmetic {
			ArithmeticType::Add => Ok(ADD_STR.to_string()),
			ArithmeticType::Sub => Ok(SUB_STR.to_string()),
			ArithmeticType::And => Ok(AND_STR.to_string()),
			ArithmeticType::Or => Ok(OR_STR.to_string()),
			ArithmeticType::Neg => Ok(NEG_STR.to_string()),
			ArithmeticType::Not => Ok(NOT_STR.to_string()),
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
