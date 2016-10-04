// #[inline(always)] -> cause these might be used in the stub

#[inline(always)]
pub fn memory_write_barrier() {
  data_memory_barrier();
}

#[inline(always)]
pub fn invalidate_caches() {

  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0360e/I1014942.html
  // Invalidate Both Caches. Also flushes the branch target cache
    unsafe{
      asm!("mcr     p15, 0, $0, c7, c7, 0"  ::"r"(0)::"volatile"
      )
    }
}

#[inline(always)]
pub fn invalidate_tlb() {

  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
  // Invalidate Inst-TLB and Data-TLB
    unsafe{
      asm!("mcr     p15, 0, $0, c8, c7, 0"  ::"r"(0)::"volatile"
      )
    }
}

#[inline(always)]
pub fn data_synchronization_barrier() {
unsafe{
  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
      asm!("MCR p15, 0, $0, c7, c10, 4"::"r"(0)::"volatile"
      )
    }
}

#[inline(always)]
pub fn data_memory_barrier() {
unsafe{
  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
      asm!("MCR p15, 0, $0, c7, c10, 5"::"r"(0)::"volatile"
      )
    }
}

#[inline(always)]
pub fn set_ttb0(page_table: *const () ) {
  /* Set Translation Table Base 0 (TTB0) */
  unsafe{
    asm!("mcr p15, 0, $0, c2, c0, 0"
          :: "r"(page_table as u32) :: "volatile"
          );

  }
}

#[inline(always)]
pub fn set_ttb1(page_table: *const () ) {
  /* Set Translation Table Base 0 (TTB0) */
  unsafe{
    asm!("mcr p15, 0, $0, c2, c0, 1"
          :: "r"(page_table as u32) :: "volatile"
          );

  }
}

#[inline(always)]
pub fn set_ttbcr(ttbcr :u32) {
	  unsafe{asm!("mcr p15, 0, $0, c2, c0, 2" : : "r" (ttbcr):: "volatile");}
}

#[inline(always)]
pub fn get_ttbcr() -> u32 {
  let mut ttbcr:u32;
	unsafe{asm!("mrc p15, 0, $0, c2, c0, 2" :  "=r" (ttbcr));}
  return ttbcr;
}


#[inline(always)]
pub fn write_domain_access_control_register(dcr :u32) {
  unsafe{
    asm!("mcr p15, 0, $0, c3, c0, 0"
          :: "r"(dcr) :: "volatile"
          );
  }
}


#[inline(always)]
pub fn get_p15_c1() -> u32{
  let mut cr : u32;
  unsafe{
    asm!("mcr p15, 0, $0, c1, c0, 0"
          : "=r"(cr) 
          );
  }
  return cr;
}

#[inline(always)]
pub fn set_p15_c1(cr : u32) {
  unsafe{
    asm!("mcr p15, 0, $0, c1, c0, 0"
          :: "r"(cr) :: "volatile"
          );
  }
}

const MMU_BIT : u32 = 1;
const DCACHE_BIT : u32 = 1<<2;
const ICACHE_BIT : u32 = 1<<12;
const XP_BIT : u32 = 1<<23;

#[inline(always)]
pub fn enable_mmu() {
  let mut cr : u32;
  cr = get_p15_c1();

  cr |= MMU_BIT;
  cr |= DCACHE_BIT;
  cr |= ICACHE_BIT;
  //extended page tables
  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0290g/Babhejba.html
  // and
  // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0360f/BGEIHGIF.html
  cr |= XP_BIT;

  set_p15_c1(cr);
}


pub fn clear_caches() {

    // TODO:
    // see http://stackoverflow.com/questions/16383007/what-is-the-right-way-to-update-mmu-translation-table
}

/* not called from stub goes here: */

const USER_MODE : u32 = 0b10000;
const FIQ_MODE : u32 = 0b10001;
const IRQ_MODE : u32 = 0b10010;
const SUPER_MODE : u32 = 0b10011;
const ABRT_MODE : u32 = 0b10111;
const UNDEF_MODE : u32 = 0b11011;
const SYS_MODE : u32 = 0b11111;
const MODE_MASK : u32 = 0b11111;

const DISABLE_FIQ : u32 = 1 << 6;
const DISABLE_IRQ : u32 = 1 << 7;

pub fn set_stack_for_modes(stack_base : ::mem::VirtualAddress) {
    unsafe {
      asm!("mov r0, $0
            mrs r1, cpsr
            bic r1, r1, $1
	          orr r1, r1, $2   /* FIQ */
            msr cpsr, r1
            mov sp, r0
            
            add r0,r0, #0x1000
            bic r1, r1, $1
	          orr r1, r1, $3  /* IRQ */
            msr cpsr, r1
            mov sp, r0
            
            add r0,r0, #0x1000
            bic r1, r1, $1
	          orr r1, r1, $4  /* ABRT */
            msr cpsr, r1
            mov sp, r0
            
            add r0,r0, #0x1000
            bic r1, r1, $1
	          orr r1, r1, $5  /* UNDEF */
            msr cpsr, r1
            mov sp, r0
            
            add r0,r0, #0x1000
            bic r1, r1, $1
	          orr r1, r1, $6  /* SYS */
            msr cpsr, r1
            mov sp, r0
            

            bic r1, r1, $1
	          orr r1, r1, $7 /* back to supervisor mode */
            msr cpsr, r1
            "
            :: 
            "r"(stack_base.0),
            "i"(MODE_MASK),
            "i"(FIQ_MODE),
            "i"(IRQ_MODE),
            "i"(ABRT_MODE),
            "i"(UNDEF_MODE),
            "i"(SYS_MODE),
            "i"(SUPER_MODE)
            : "sp","r0","r1" : "volatile"
      )
    }
}
