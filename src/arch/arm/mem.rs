// #[inline(always)] -> cause these might be used in the stub

// contants are auto inlined: https://doc.rust-lang.org/book/const-and-static.html
pub const PAGE_SIZE : usize = 4096;
pub const PAGE_MASK : usize = PAGE_SIZE - 1;

#[inline(always)]
pub fn memory_write_barrier() {

  // flush every cache i can think of..
  // flush v3/v4 cache and flush v4 TLB
    unsafe{
      asm!("mov     r0, #0
            mcr     p15, 0, r0, c7, c7, 0
            mcr     p15, 0, r0, c8, c7, 0 "
            :::"rdi": "volatile"
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
pub fn enable_mmu() {
  let mut cr : u32;
  unsafe{
    asm!("mcr p15, 0, $0, c1, c0, 0"
          : "=r"(cr) 
          );
  }

  cr |= 1; // mmu
  // TODO enabled caches at somepoint..
  // cr |= (1<<2 ); // dcache
  // cr |= (1<<12); // icache

  unsafe{
    asm!("mcr p15, 0, $0, c1, c0, 0"
          :: "r"(cr) :: "volatile"
          );

  }
}


pub fn clear_caches() {

    // TODO:
    // see http://stackoverflow.com/questions/16383007/what-is-the-right-way-to-update-mmu-translation-table
}
