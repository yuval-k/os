use core::slice;

/*
based on:

.globl vector_start, vector_end
vector_start
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	nop
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]

	.word	vector_reset
	.word	vector_undefined
	.word	vector_softint
	.word	vector_prefetch_abort
	.word	vector_data_abort
	.word	0
	.word	vector_irq
	.word	vector_fiq
vector_end:

*/ 
pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0xea00_0000);

 
#[naked]
 fn vector_table_asm() {
    unsafe {
        asm!("ldr pc, [pc, #24]" : : : : "volatile");
    };
}

// TODO change vec_table to point to the right place in memory
pub fn build_vector_table() {
    unsafe {
        let mut vec_table : &'static mut [u32] = slice::from_raw_parts_mut(VECTORS_ADDR.0 as *mut u32, 4*8*2);

        let asmjump : *const u32 = vector_table_asm as *const u32;
        vec_table[0] = *asmjump;
        vec_table[1] = *asmjump;
        vec_table[2] = *asmjump;
        vec_table[3] = *asmjump;
        vec_table[4] = *asmjump;
        vec_table[5] = 0;
        vec_table[6] = *asmjump;
        vec_table[7] = *asmjump;

        // default implementations
        vec_table[8+0] = vector_reset as u32;
        vec_table[8+1] = vector_undefined as u32;
        vec_table[8+2] = vector_softint as u32;
        vec_table[8+3] = vector_prefetch_abort as u32;
        vec_table[8+4] = vector_data_abort as u32;
        vec_table[8+5] = 0;
        vec_table[8+6] = vector_irq as u32;
        vec_table[8+7] = vector_fiq as u32;

    }

// 

}

extern "C" fn vector_reset() {
    loop{}
}

extern "C" fn vector_undefined() {
    loop{}
}

extern "C" fn vector_softint() {
    loop{}
}

extern "C" fn vector_prefetch_abort() {
    loop{}
}

extern "C" fn vector_data_abort() {
    loop{}
}

extern "C" fn vector_irq() {
    loop{}
}

extern "C" fn vector_fiq() {
    loop{}
}
