
// small boot stack, when mmu is enabled gonna get a better one..
#define STACK_SIZE 0x800
#define L1_TABLE_ALIGN 14
#define L1_TABLE_SIZE 4*0x1000
#define L2_TABLE_SIZE 4*0x100



#define GPFSELONE      0x3F200000
#define GPFSELTWO      0x3F200008

#define GPFSELBASE      0x3F20

.section .stub


_start:
    .globl _start
    .globl stub_l1pagetable
    .globl stub_l2pagetable




 @ enable jtag
    mov r1, #0x3F
    lsl r1,r1, #8
    orr r1, #0x20
    lsl r1,r1, #16
    mov r2, r1
    orr r2, r2, #0x94
    mov r3, r1
    orr r3, r3, #0x98

    mov r4, #0
    str r4, [r2]

    mov r4, #150
    1: subs r4,r4, #1
    bne 1b


    mov r4, #0
    @ r4 = (1<<4)|(1<<22)|(1<<24)|(1<<25)|(1<<27)
    orr r4,r4, #0xB4
    lsl r4, r4, #20
    orr r4,r4, #0x10
    str r4, [r3]

    mov r4, #150
    1: subs r4,r4, #1
    bne 1b


    mov r4, #0
    str r4, [r3]



    ldr r0, [r1]
    bic r0, #0x7000; //gpio4
    orr r0, #0x2000
    str r0, [r1]

    add r1, r1, #8

    ldr r0, [r1]
    bic r0, #0x1C0 //gpio22
    orr r0, #0xC0
    bic r0, #0x7000 //gpio24
    orr r0, #0x3000
    bic r0, #0x38000 //gpio25
    orr r0, #0x18000
    bic r0, #0xE00000 //gpio27
    orr r0, #0x600000
    str r0, [r1]

    @ wait for debugger
    @ b .


    ldr sp, =temp_stub_stack @ Set up the stack
    b stub_main @ Jump to the main function
1: 
    b 1b @ Halt

_secondary_start:
    .globl _secondary_start
    
    ldr sp, =temp_stub_stack @ Set up the stack
    b stub_secondary_core @ Jump to the main function
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
