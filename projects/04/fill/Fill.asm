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
  @color
  M=-1
  @FILL_SCREEN
  0;JMP

(SET_WHITE)
  @color
  M=0

// Fill screen with color specified in variable
(FILL_SCREEN)
  // Write 256*32
  @8192
  D=A
  @block_num
  M=D
  @i
  M=0

// Write color to offset i
(WRITE_BLOCK)
  // if loop variable equals or is greater than block_num we loop from beginning
  @i
  D=M
  @block_num
  D=M-D
  @CHECK_INPUT
  D; JLE
  // Write block
  @i
  D=M
  @SCREEN
  D=A+D
  @offset
  M=D
  @color
  D=M
  @offset
  A=M
  M=D // set color to offset address

  // increment loop index
  @i
  M=M+1
  // write next block
  @WRITE_BLOCK
  0; JMP
