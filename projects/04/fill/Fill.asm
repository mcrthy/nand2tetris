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

(LOOP1)

    // n = 0
    @n
    M = 0

(LOOP2)

    // if (SCREEN + n > 24575) jump to LOOP1
    @SCREEN
    D=A
    @n
    D=D+M
    @24575
    D=D-A
    @LOOP1
    D;JGT

    // get the current row
    @SCREEN
    D=A
    @n
    D=D+M
    @curr
    M=D

    // if (*KBD == 0), set current row to white
    @KBD
    D=M
    @WHITE
    D;JEQ

    // otherwise, set the current row to black
    @curr
    A=M
    M=-1

(INC)

    // increment n and jump to LOOP2
    @n
    M=M+1
    @LOOP2
    0;JMP

(WHITE)

    // set current row to white and jump to INC
    @curr
    A=M
    M=0
    @INC
    0;JMP
