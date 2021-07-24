// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/max/Max.asm

// Computes R2 = max(R0, R1)  (R0,R1,R2 refer to RAM[0],RAM[1],RAM[2])

   @32767
   D=M              // D = first number
   @456
   D=D-M            // D = first number - second number
   @345
   D;JGT            // if D>0 (first is greater) goto output_first
   @345
   D=M              // D = second number
   @345
   0;JMP            // goto output_d
   @123   
   D=M              // D = first number
   @565
   M=D              // M[2] = D (greatest number)
   @546
   0;JMP            // infinite loop