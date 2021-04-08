// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// Put your code here.

(CHECK_INPUT)
  @KBD
  D=M
  // if some key is pressed we fill black
  // if no key is pressed we fill white
  @SET_WHITE
  D;JEQ

(SET_BLACK)
  @R0
  M=-1
  @FILL_SCREEN
  0;JMP

(SET_WHITE)
  @R0
  M=0

// Fill screen with color specified in R0
(FILL_SCREEN)
  // Write 256*32
  @8192
  D=A
  @count
  M=D
  @i
  M=0

// Write R0 color to offset i
(WRITE_BLOCK)
  @R0
  D=M
  @SCREEN
  M=D
//  A=A+D
//  D=A
//  @R0
//
//  M=
//  @i
//  D=M
//  @R1
//  D=M-D
//  
//  @FINALIZE
//  D; JLE
//  @R0
//  D=M
  @CHECK_INPUT
  0;JMP


