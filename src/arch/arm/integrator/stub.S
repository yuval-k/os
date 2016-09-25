interrupt_vector_table:
    b . @ Reset
    b . 
    b . @ SWI instruction
    b . 
    b .
    b .
    b .
    b .

#define STACK_SIZE 0x1000

.comm stack, STACK_SIZE @ Reserve 64k stack in the BSS
_start:
    .globl _start
    ldr sp, =stack+STACK_SIZE @ Set up the stack
    bl rust_main @ Jump to the main function
1: 
    b 1b @ Halt
