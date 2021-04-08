// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

// Put your code here.
  // init loop variables
  @i
  M=0
  @sum
  M=0
(MUL_LOOP)
  // if loop variable equals or is greater than R1 we exit loop
  @i
  D=M
  @R1
  D=M-D
  @FINALIZE
  D; JLE
  // Add R0 once
  @R0
  D=M
  @sum
  M=M+D
  // increment loop index
  @i
  M=M+1
  // loop again
  @MUL_LOOP
  0; JMP

(FINALIZE)
  // set sum to output register
  @sum
  D=M
  @R2
  M=D
(END)
  @END
  0; JMP
