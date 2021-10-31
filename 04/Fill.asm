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

// SET COLOR(16) BLACK
@18
M=-1

@DRAW
D; JGT  // キーが押されていないなら色を白にする処理を跳ばす

// SET COLOR(16) WHITE
@18
M=0

(DRAW)
@16384
D=A
@17
M=D

@8192  // 繰り返しのカウンタの用意
D=A
@I
M=D

(DRAWLOOP)
@I  // ループのカウント
M=M-1
@18  // 塗りつぶす色をセット
D=M
@17  // 線を描画
A=M
M=D

@I
D=M
@LOOP  // 描画終了
D;JEQ

@17  // 次に描画する位置をセット
D=M
@1
D=D+A
@17 
M=D
@DRAWLOOP
0;JMP
