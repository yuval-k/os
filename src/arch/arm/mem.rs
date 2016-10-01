// #[inline(always)] -> cause these might be used in the stub

// contants are auto inlined: https://doc.rust-lang.org/book/const-and-static.html
pub const PAGE_SIZE : usize = 4096;
pub const PAGE_MASK : usize = PAGE_SIZE - 1;
// 4096 entries of 1MB each (=4gb address space). each entry is 4 bytes.
pub const L1TABLE_ENTRIES : usize = 4096; 


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
pub fn disable_access_checks() {
  unsafe{
    asm!("mcr p15, 0, $0, c3, c0, 0"
          :: "r"(3) :: "volatile"
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

#[inline(always)]
pub fn enable_mmu() {
  let mut cr : u32;
  cr = get_p15_c1();

  cr |= MMU_BIT;
  cr |= DCACHE_BIT;
  cr |= ICACHE_BIT;

  set_p15_c1(cr);
}


pub fn clear_caches() {

    // TODO:
    // see http://stackoverflow.com/questions/16383007/what-is-the-right-way-to-update-mmu-translation-table
}
