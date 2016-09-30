
// small boot stack, when mmu is enabled gonna get a better one..
#define STACK_SIZE 0x800
#define L1_TABLE_ALIGN 14
#define L1_TABLE_SIZE 4*0x1000

.section .stub

_start:
    .globl _start
    .globl l1pagetable
    ldr sp, =temp_stub_stack @ Set up the stack
    bl integrator_main @ Jump to the main function
1: 
    b 1b @ Halt

.ltorg
.section .stub.data
.align 4
.space STACK_SIZE
temp_stub_stack:
.space 4
.align L1_TABLE_ALIGN
l1pagetable:
.space L1_TABLE_SIZE
