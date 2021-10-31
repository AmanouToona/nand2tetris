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

(LOOP)
@24576
D=M
@STARTBLACKOUT
D; JGT

// WHITE OUT SCREEN
@16384
D=A
@17
M=D

@8192
D=A
@I
M=D
(WHITEOUTLOOP)
@I
M=M-1
@17
A=M
M=0  // DRAW WHITE LINE
@I
D=M

@LOOP  // END WHITEOUT
D;JEQ

// SET NEXT POSITION
@17
D=M
@1
D=D+A
@17 
M=D
@WHITEOUTLOOP
0;JMP


// BLACK OUT SCREEN
(STARTBLACKOUT)
@16384
D=A
@17
M=D

@8192
D=A
@I
M=D
(BLACKOUTLOOP)
@I
M=M-1
@17
A=M
M=-1  // DRAW BLACK LINE
@I
D=M

@LOOP  // END BLACKOUT
D;JEQ

// SET NEXT POSITION
@17
D=M
@1
D=D+A
@17
M=D
@BLACKOUTLOOP
0;JMP




