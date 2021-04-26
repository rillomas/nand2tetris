// Init SP
@16
D=A
@SP
M=D
// push 7
@7
D=A
// assign value to SP address
@SP
A=M
M=D
// advance SP
@SP
M=M+1
// push 8
@8
D=A
@SP
A=M
M=D
@SP
M=M+1
// add
@SP
A=M
// store previous
A=A-1
D=M
// add current and previous
A=A-1
M=D+M
// Update SP
D=A+1
@SP
M=D