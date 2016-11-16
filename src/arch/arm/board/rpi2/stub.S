
// small boot stack, when mmu is enabled gonna get a better one..
#define STACK_SIZE 0x800
#define L1_TABLE_ALIGN 14
#define L1_TABLE_SIZE 4*0x1000
#define L2_TABLE_SIZE 4*0x100

.section .stub

_start:
    .globl _start
    .globl stub_l1pagetable
    .globl stub_l2pagetable
    ldr sp, =temp_stub_stack @ Set up the stack
    bl stub_main @ Jump to the main function
1: 
    b 1b @ Halt

_secondary_start:
    .globl _secondary_start
    
    ldr sp, =temp_stub_stack @ Set up the stack
    bl stub_secondary_core @ Jump to the main function
1: 
    b 1b @ Halt

.ltorg
.section .stub.data
.align 4
.space STACK_SIZE
temp_stub_stack:


.section .stub.data.pagegtable
.align L1_TABLE_ALIGN
stub_l1pagetable:
.space L1_TABLE_SIZE
stub_l2pagetable:
.space L2_TABLE_SIZE
