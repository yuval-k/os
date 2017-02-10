
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




 @ enable jtag
 @ r1 = 0x20200000
    mov r1, #0x20
    lsl r1, r1, #8
    orr r1, #0x20
    lsl r1, r1, #16
    @ r1 is now 0x20200000

    ldr r0, [r1]
    bic r0, #0x7000; //gpio4
    orr r0, #0x2000
    str r0, [r1]

    add r1, r1, #8

    ldr r0, [r1]
    bic r0, #0x0001C0 //gpio22
    orr r0, #0x0000C0
    bic r0, #0x007000 //gpio24
    orr r0, #0x003000
    bic r0, #0x038000 //gpio25
    orr r0, #0x018000
    bic r0, #0xE00000 //gpio27
    orr r0, #0x600000
    str r0, [r1]

    @ wait for debugger (the debugger should set r0 to zero so we exit the loop)
    mov r0, #31
    1: cmp r0, #0
    bne 1b

    ldr sp, =temp_stub_stack @ Set up the stack
    bl stub_main @ Jump to the main function
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
