use core::slice;
use core::ops::{Index, IndexMut};
use core::cmp;
use core::ops;

use super::cpu;
use ::mem::FrameAllocator;
use ::mem::MemorySize;
use ::sync;

// contants are auto inlined: https://doc.rust-lang.org/book/const-and-static.html
pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;
// 4096 entries of 1MB each (=4gb address space). each entry is 4 bytes.
pub const L1TABLE_ENTRIES: usize = 4096;
pub const L2TABLE_ENTRIES: usize = 256;


pub const MB_SHIFT: usize = 20;
pub const MB_SIZE: usize = 1 << MB_SHIFT;
pub const MB_MASK: usize = MB_SIZE - 1;

pub struct LameFrameAllocator {
    cpu_mutex : sync::CpuMutex<LameFrameAllocatorInner>,
}

pub struct LameFrameAllocatorInner {
    nextfree: ::mem::PhysicalAddress,
    max: ::mem::PhysicalAddress,

    ranges: [Option<ops::Range<::mem::PhysicalAddress>>;10],
    free_frames: [Option<ops::Range<::mem::PhysicalAddress>>; 10],
    free_frames_index: usize,
}

impl ::mem::FrameAllocator for LameFrameAllocator {
    fn allocate(&self, number: usize) -> Option<::mem::PhysicalAddress> {
        self.cpu_mutex.lock().allocate(number)
    }
    fn deallocate(&self, addr: ::mem::PhysicalAddress, size: usize) {
        self.cpu_mutex.lock().deallocate(addr, size)
    }
}

impl LameFrameAllocator{
    pub fn new(ranges: &[ops::Range<::mem::PhysicalAddress>],
               max_size: usize)
               -> LameFrameAllocator{
        let mut copy_ranges : [Option<ops::Range<::mem::PhysicalAddress>>;10] =  [None, None, None, None, None, None, None, None, None, None];
        let mut i = 0;
        for r in ranges {
            copy_ranges[i] = Some(r.clone());
            i += 1;
        }

        LameFrameAllocator{cpu_mutex: sync::CpuMutex::new(
            LameFrameAllocatorInner {
                max: ::mem::PhysicalAddress(max_size),
                nextfree: ::mem::PhysicalAddress(PAGE_SIZE), /* don't allocate frame zero cause the vector table is there.. */
                ranges: copy_ranges,
                free_frames: [None, None, None, None, None, None, None, None, None, None],
                free_frames_index: 0,
                        }
                )
        }
    }
}

impl LameFrameAllocatorInner {
    // assume no interrupts here.
    fn allocate(&mut self, number: usize) -> Option<::mem::PhysicalAddress> {
        if self.nextfree >= self.max {
            return None;
        }

        // if (number == 1) && self.free_frames_index > 0 {
        // self.free_frames_index -= 1;
        // if let Some(frameRange) =  self.free_frames[self.free_frames_index] {
        //
        // TODO if range is bigger than one page take just one page.
        // if frameRange.
        //
        // self.free_frames[self.free_frames_index] = None;
        // return  Some(frame);
        // } else {
        // panic!("Frame not there even though it should");
        // }
        // }
        //
        let mut cur_free;
        let mut potential_next;

        'outer: loop {
            cur_free = self.nextfree;

            potential_next = cur_free.offset((number << PAGE_SHIFT) as isize);

            let cur_range = cur_free..potential_next;

            for mayber in &self.ranges {
                if let Some(ref r) = *mayber {
                    if (cur_range.start < r.end) && (r.start < cur_range.end) {
                        self.nextfree = cmp::max(self.nextfree, r.end);
                        continue 'outer;
                    }
                }
            }

            break;
        }

        self.nextfree = potential_next;

        if self.nextfree > self.max {
            return None;
        }


        Some(cur_free)
    }

    fn deallocate(&mut self, addr: ::mem::PhysicalAddress, size: usize) {
        if (self.free_frames_index + 1) < self.free_frames.len() {
            self.free_frames[self.free_frames_index] = Some(addr..addr.uoffset(size << PAGE_SHIFT));
            self.free_frames_index += 1;
        }
    }

}

#[repr(packed)]
pub struct L2TableDescriptor(u32);

#[repr(packed)]
pub struct L1TableDescriptor(u32);

pub struct FirstLevelTableDescriptor(u32);

// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0333h/Babifihd.html
const CACHEABLE: u32 = 1 << 3;
const BUFFERABLE: u32 = 1 << 2;
// const L2_SHAREABLE  : u32 = 1 << 10;

// const L2_NX  : u32 = 1;
const L2_XPAGE_TYPE: u32 = 1 << 1;

// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0211k/Caceaije.html
// read write all:
const L2_AP_ALL_ACCESS: u32 = 0b11 << 4;
const L1_AP_ALL_ACCESS: u32 = 0b11 << 10;

// where we gonna map the virt table itself
const L1_VIRT_ADDRESS: ::mem::VirtualAddress = ::mem::VirtualAddress(0xe000_0000);

impl L1TableDescriptor {
    fn new(physical_address_of_l2: ::mem::PhysicalAddress) -> L1TableDescriptor {

        if physical_address_of_l2.0 & PAGE_MASK != 0 {
            panic!("Can't map unaligned l2 frames")
        }

        let mut d: L1TableDescriptor = L1TableDescriptor(0);
        // 4kb page
        d.0 |= 1;

        // set permissions
        d.0 |= physical_address_of_l2.0 as u32;

        d
    }

    fn new_section(section_addr: ::mem::PhysicalAddress, cachable: bool) -> L1TableDescriptor {

        if section_addr.0 & MB_MASK != 0 {
            panic!("Can't map unaligned sections")
        }

        let mut d: L1TableDescriptor = L1TableDescriptor(0);
        // 1MB section
        d.0 |= 0b10;

        if cachable {
            d.0 |= CACHEABLE;
            d.0 |= BUFFERABLE;
        }

        d.0 |= L1_AP_ALL_ACCESS;

        d.0 |= section_addr.0 as u32;

        d
    }

    fn is_present(&self) -> bool {
        self.0 != 0
    }

    fn is_section(&self) -> bool {
        self.0 & 0b11 == 0b10
    }

    fn get_physical_address(&self) -> ::mem::PhysicalAddress {
        if !self.is_present() {
            panic!("entry not present!")
        }
        if self.is_section() {
            return ::mem::PhysicalAddress((self.0 as usize) & (!MB_MASK));
        }
        ::mem::PhysicalAddress((self.0 as usize) & (!PAGE_MASK))
    }
}

impl L2TableDescriptor {
    fn new(physical_address_of_page: ::mem::PhysicalAddress) -> L2TableDescriptor {
        if (physical_address_of_page.0 & PAGE_MASK) != 0 {
            panic!("Can't map unaligned l2 frames")
        }

        let mut d: L2TableDescriptor = L2TableDescriptor(0);
        // 4kb page
        d.0 |= L2_XPAGE_TYPE;
        d.0 |= CACHEABLE;
        d.0 |= BUFFERABLE;
        d.0 |= L2_AP_ALL_ACCESS;

        // Only one cpu now.. no need to set shareable

        // set permissions

        d.0 |= physical_address_of_page.0 as u32;

        d
    }

    fn new_device(physical_address_of_page: ::mem::PhysicalAddress) -> L2TableDescriptor {
        if (physical_address_of_page.0 & PAGE_MASK) != 0 {
            panic!("Can't map unaligned l2 frames")
        }

        let mut d: L2TableDescriptor = L2TableDescriptor(0);
        // 4kb page
        d.0 |= L2_XPAGE_TYPE;
        d.0 |= L2_AP_ALL_ACCESS;

        // Only one cpu now.. no need to set shareable
        // set permissions
        d.0 |= physical_address_of_page.0 as u32;

        d
    }

    fn is_present(&self) -> bool {
        self.0 != 0
    }

    fn get_physical_address(&self) -> ::mem::PhysicalAddress {
        if !self.is_present() {
            panic!("entry not present!")
        }
        ::mem::PhysicalAddress((self.0 as usize) & (!PAGE_MASK))
    }
}
// repr C might not be needed, but let's be on the safe side.
#[repr(packed)]
pub struct L1Table {
    pub descriptors: &'static mut [L1TableDescriptor],
}
#[repr(packed)]
pub struct L2Table {
    pub descriptors: &'static mut [L2TableDescriptor],
}

impl Index<usize> for L1Table {
    type Output = L1TableDescriptor;

    fn index(&self, index: usize) -> &L1TableDescriptor {
        &self.descriptors[index]
    }
}

impl IndexMut<usize> for L1Table {
    fn index_mut(&mut self, index: usize) -> &mut L1TableDescriptor {
        &mut self.descriptors[index]
    }
}


impl Index<usize> for L2Table {
    type Output = L2TableDescriptor;

    fn index(&self, index: usize) -> &L2TableDescriptor {
        &self.descriptors[index]
    }
}

impl IndexMut<usize> for L2Table {
    fn index_mut(&mut self, index: usize) -> &mut L2TableDescriptor {
        &mut self.descriptors[index]
    }
}

// The stub has provided us with an l1 table and an aligned buffer that we can use for l2 table that are identity mapped.
//
// we are going to use those to the l2 buffer to map our new page tabled, initialize with the kernel,
// stack and switch to it.
//
//
pub struct MemLayout {
    pub kernel_start_phy: ::mem::PhysicalAddress,
    pub kernel_start_virt: ::mem::VirtualAddress,
    pub kernel_end_virt: ::mem::VirtualAddress,
    pub stack_phy: ::mem::PhysicalAddress,
    pub stack_virt: ::mem::VirtualAddress,
}


fn get_init_frames(fa: & ::mem::FrameAllocator) -> [::mem::PhysicalAddress; 5] {
    const NUM_FRAMES: usize = 7; // guaranteed to have somthing aligned here..
    let mut free_frames: [::mem::PhysicalAddress; NUM_FRAMES] = [::mem::PhysicalAddress(0); NUM_FRAMES];
    let pa = fa.allocate(NUM_FRAMES).unwrap();

    // find out which one devides with 16k
    let l1_start_frame = (4 - ((pa.0 >> PAGE_SHIFT) & 0b11)) & 0b11;

    for i in 0..NUM_FRAMES {
        let shifted_index = (i + l1_start_frame) % NUM_FRAMES;
        free_frames[i] = pa.offset((shifted_index << PAGE_SHIFT) as isize);
    }

    // don't need the last two..
    fa.deallocate(free_frames[5], 1);
    fa.deallocate(free_frames[6], 1);

    return [free_frames[0], free_frames[1], free_frames[2], free_frames[3], free_frames[4]];
}

fn up(a: usize) -> usize {
    (a + PAGE_MASK) & (!PAGE_MASK)
}
fn down(a: usize) -> usize {
    a & (!PAGE_MASK)
}

fn mb_up(a: usize) -> usize {
    (a + MB_MASK) & (!MB_MASK)
}
fn mb_down(a: usize) -> usize {
    a & (!MB_MASK)
}


// TODO fix frame allocator to not use stub and stack.
pub fn init_page_table(l1table_identity: ::mem::VirtualAddress,
                       l2table_identity: ::mem::VirtualAddress,
                       ml: &MemLayout,
                       fa: &mut ::mem::FrameAllocator)
                       -> PageTable {
    let mut active_table = unsafe { L1Table::from_virt_address_no_init(l1table_identity) };
    let mut l2 = unsafe { L2Table::from_virt_address_init(l2table_identity) };

    // get some frames, where the first four are contingous and aligned to 16kb:
    let free_frames: [::mem::PhysicalAddress; 5] = get_init_frames(fa);

    // first 4 frames are for l1 table (as they are on 16k boundery).
    // get the 5th frame to use as temporary coarse table

    // map our new page table to memory so we can write to it.
    active_table[L1_VIRT_ADDRESS.0 >> MB_SHIFT] =
        L1TableDescriptor::new(::mem::PhysicalAddress(l2table_identity.0));

    // map the l1 page table
    l2[0] = L2TableDescriptor::new(free_frames[0]);
    l2[1] = L2TableDescriptor::new(free_frames[1]);
    l2[2] = L2TableDescriptor::new(free_frames[2]);
    l2[3] = L2TableDescriptor::new(free_frames[3]);
    // map one more frame so we can use map the table itself
    l2[4] = L2TableDescriptor::new(free_frames[4]);

    let next_free_l2_index = 5;

    // flush changes
    cpu::memory_write_barrier();
    cpu::flush_caches();
    cpu::invalidate_tlb();
    // TODO: if loading code, need to do an ISB (on arm)

    // our blank l1 and l2 mapped pages should be available now.
    let mut newl1 = unsafe { L1Table::from_virt_address_init(L1_VIRT_ADDRESS) };
    // l2 is the fifth frame..
    let mut newl2 =
        unsafe { L2Table::from_virt_address_init(L1_VIRT_ADDRESS.offset((4 << PAGE_SHIFT) as isize)) };

    // map the new map in itself in the same address - so we can access it aftet we switch tables..
    newl1[L1_VIRT_ADDRESS.0 >> MB_SHIFT] = L1TableDescriptor::new(free_frames[4]); //point to the l2 descriptor
    newl2[0] = L2TableDescriptor::new(free_frames[0]);
    newl2[1] = L2TableDescriptor::new(free_frames[1]);
    newl2[2] = L2TableDescriptor::new(free_frames[2]);
    newl2[3] = L2TableDescriptor::new(free_frames[3]);
    newl2[4] = L2TableDescriptor::new(free_frames[4]);

    // now when we will switch the page table, the page table itself will be available in the same place.


    // map the kernel in the new page table:
    let kernel_size = up(::mem::to_bytes(ml.kernel_end_virt - ml.kernel_start_virt));
    // mega bytes rounded up
    let nummb = ((kernel_size + MB_MASK) & (!MB_MASK)) >> MB_SHIFT;

    // for each meg:
    for i in 0..nummb {

        // get new frame
        let frame = fa.allocate(1).unwrap();

        // map the frame
        l2[next_free_l2_index] = L2TableDescriptor::new(frame);

        // flush changes
        // TODO not sure i need all of those.. but lets start with that
        cpu::memory_write_barrier();
        // make sure all is in main memory.. probably not needed now, but good practice for smp..
        cpu::flush_caches();
        cpu::invalidate_tlb();

        // frame now available here:
        let frame_address = L1_VIRT_ADDRESS.uoffset(next_free_l2_index << PAGE_SHIFT);
        let mut curr_kernel_l2 = unsafe { L2Table::from_virt_address_init(frame_address) };
        // for each 4k block in the mb, map it in new_framel2
        let curphy_start = ml.kernel_start_phy.uoffset(i << MB_SHIFT);
        // check that in the end we don't map a full MB
        let nextmb = ml.kernel_start_phy.uoffset((i + 1) << MB_SHIFT);
        let curphy_end = if (i + 1) == nummb {
            ml.kernel_start_phy.uoffset(kernel_size)
        } else {
            nextmb
        };

        let mut l2loopindex = 0;
        for cur_frame in (curphy_start.0..curphy_end.0).step_by(PAGE_SIZE) {
            curr_kernel_l2[l2loopindex] = L2TableDescriptor::new(::mem::PhysicalAddress(cur_frame));
            l2loopindex += 1;
        } // TODO: why do i need the code below?
        for cur_frame in (curphy_end.0..nextmb.0).step_by(PAGE_SIZE) {
            curr_kernel_l2[l2loopindex] = L2TableDescriptor::new(::mem::PhysicalAddress(cur_frame));
            l2loopindex += 1;
        }

        // add the l2 frame to the l2 map
        newl1[(ml.kernel_start_virt.0 >> MB_SHIFT) + i] = L1TableDescriptor::new(frame);

    }
    // map the stack
    // get stack pointer
    let sp = ml.stack_virt.0 & (!PAGE_MASK);
    let spframe = ml.stack_phy.0 & (!PAGE_MASK);

    let mut need_init = false;

    if !newl1[sp >> MB_SHIFT].is_present() {
        let frame = fa.allocate(1).unwrap();
        need_init = true;
        newl1[sp >> MB_SHIFT] = L1TableDescriptor::new(frame);
    }
    // first, map the existing current stack
    let stackframe = newl1[sp >> MB_SHIFT].get_physical_address();

    // replaceing page - destroy old page in caches. TODO - is this needed.
    cpu::memory_write_barrier();
    cpu::flush_caches();
    cpu::invalidate_tlb();

    // temporary the l2 entry to memory
    l2[next_free_l2_index] = L2TableDescriptor::new(stackframe);

    // flush changes
    cpu::memory_write_barrier();
    cpu::flush_caches();
    cpu::invalidate_tlb();

    let frame_address = L1_VIRT_ADDRESS.uoffset(next_free_l2_index << PAGE_SHIFT);
    let mut stack_l2 = unsafe { if need_init {  L2Table::from_virt_address_init(frame_address) } else { L2Table::from_virt_address_no_init(frame_address) } };
    // TODO: set nx bit
    // TODO: make stack size a constant and not hard coded
    stack_l2[(sp >> PAGE_SHIFT) & 0xFF] = L2TableDescriptor::new(::mem::PhysicalAddress(spframe));
    stack_l2[((sp >> PAGE_SHIFT) & 0xFF) + 1] = L2TableDescriptor::new(::mem::PhysicalAddress(spframe + PAGE_SIZE));


    // turn on new mmu and free the stub memory
    // the kernel now has a page table with the l1 mapped to
    // L1_VIRT_ADDRESS and l2 table that maps the virt table to L1_VIRT_ADDRESS mapping is at
    // L1_VIRT_ADDRESS + 5*PAGE_SIZE. it has five entries taken. so available from index #5
    // so once can use that 5th index to init new frames and place them in the page table.

    // when memory is freed, we need to find out the physical addresses so we can free them.
    // to do that we will need to map the l2 table, and temporary map it again.
    // in read all the frames it points to and free them.
    cpu::memory_write_barrier();
    // disable access checks for domain 0
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
    // set domain 0 to what ever is in the table.
    cpu::flush_caches();
    cpu::data_synchronization_barrier();
    cpu::write_domain_access_control_register(1);
    cpu::set_ttb0(free_frames[0].0 as *const ());
    cpu::invalidate_tlb();

    PageTable{
        cpu_mutex : sync::CpuMutex::new(PageTableInner {
            descriptors: newl1,
            tmp_map: newl2,
            }
        )
    }
}

pub struct PageTable {
    cpu_mutex : sync::CpuMutex<PageTableInner>,
}

pub struct PageTableInner {
    pub descriptors: L1Table,
    tmp_map: L2Table,
}

impl PageTableInner {
    // TODO add error handling..
    fn map_single(&mut self,
                  frameallocator: & ::mem::FrameAllocator,
                  p: ::mem::PhysicalAddress,
                  v: ::mem::VirtualAddress) {
        self.map_single_descriptor(frameallocator, L2TableDescriptor::new(p), v)
    }

    fn map_section(&mut self, s: L1TableDescriptor,
                              v: ::mem::VirtualAddress) {
        let l1_index = v.0 >> MB_SHIFT;
        self.descriptors[l1_index] = s;
    }

    fn map_single_descriptor(&mut self,
                             frameallocator: & ::mem::FrameAllocator,
                             p: L2TableDescriptor,
                             v: ::mem::VirtualAddress) {
        let l1_index = v.0 >> MB_SHIFT;
        let mut new_frame: bool = false;
        // get physical addresss
        // temporary map it to here using the active page table
        if !self.descriptors[l1_index].is_present() {
            let frame = frameallocator.allocate(1).unwrap();
            self.descriptors[l1_index] = L1TableDescriptor::new(frame);
            new_frame = true;
        }

        let l2phy = self.descriptors[l1_index].get_physical_address();

        // SHOULD WE FLUSH CASHES HERE?

        // 0-3 are page table itself
        // 4 is the tmp map itself
        // 5 is free!
        const FREE_INDEX: usize = 5;
        self.tmp_map[FREE_INDEX] = L2TableDescriptor::new(l2phy);

        // TODO: find the frame for l2, and temporary map it..
        // and add teh mapping

        cpu::memory_write_barrier();
        cpu::flush_caches();
        cpu::invalidate_tlb();
        cpu::data_synchronization_barrier();

        let mapped_address = L1_VIRT_ADDRESS.offset((FREE_INDEX * PAGE_SIZE) as isize);
        
        // frame now available here:
        let mut l2_for_phy = unsafe { if new_frame {  L2Table::from_virt_address_init(mapped_address) } else { L2Table::from_virt_address_no_init(mapped_address) } };

        let l2_index = (v.0 >> PAGE_SHIFT) & 0xFF;

        l2_for_phy[l2_index] = p;

        cpu::memory_write_barrier();
        cpu::flush_caches();
        cpu::invalidate_tlb();
        // page should be mapped now
    }

    fn p2v(&mut self, p: ::mem::PhysicalAddress) -> Option<::mem::VirtualAddress> {
        let l1table = self.descriptors.descriptors.iter();
        for (index, l1desc) in l1table.enumerate() {
            if !l1desc.is_present() {
                continue;
            }

            if l1desc.is_section() {
                let phy_mb = l1desc.get_physical_address();
                if mb_down(p.0) == phy_mb.0 {
                    return Some(::mem::VirtualAddress((index << MB_SHIFT) + (p.0 & MB_MASK)));
                }
                continue;
            }

            // not a section..

            let l2phy = l1desc.get_physical_address();

            // TODO!
            const FREE_INDEX: usize = 5;
            self.tmp_map[FREE_INDEX] = L2TableDescriptor::new(l2phy);

            cpu::memory_write_barrier();
            // wait for the data to arrive to physical memory
            cpu::data_synchronization_barrier();
            cpu::flush_caches();
            cpu::invalidate_tlb();

            let mapped_address = L1_VIRT_ADDRESS.uoffset(FREE_INDEX * PAGE_SIZE);
            let l2_for_phy = unsafe { L2Table::from_virt_address_no_init(mapped_address) };
            for j in 0..l2_for_phy.descriptors.len() {
                if l2_for_phy[j].is_present() {
                    let phy = l2_for_phy.descriptors[j].get_physical_address();
                    if down(p.0) == phy.0 {
                        return Some(::mem::VirtualAddress((index << MB_SHIFT) + (j << PAGE_SHIFT) +
                                                          (p.0 & PAGE_MASK)));
                    }
                }
            }
        }
        None
    }

    fn v2p(&mut self, v: ::mem::VirtualAddress) -> Option<::mem::PhysicalAddress> {
        let l1_index = v.0 >> MB_SHIFT;
        let l1descriptor = &self.descriptors[l1_index];
        if !l1descriptor.is_present() {
            return None;
        }

        if l1descriptor.is_section() {
            let phy_mb = l1descriptor.get_physical_address();
            let p = ::mem::PhysicalAddress(phy_mb.0 | (v.0 & MB_MASK));
            return Some(p);
        }


        // map the l2 table
        const FREE_INDEX: usize = 5;
        self.tmp_map[FREE_INDEX] = L2TableDescriptor::new(l1descriptor.get_physical_address());

        cpu::memory_write_barrier();
        cpu::flush_caches();
        cpu::invalidate_tlb();
        cpu::data_synchronization_barrier();

        let mapped_address = L1_VIRT_ADDRESS.uoffset(FREE_INDEX << PAGE_SHIFT);
        // frame now available here:
        let l2_for_phy = unsafe { L2Table::from_virt_address_no_init(mapped_address) };

        let l2_index = (v.0 >> PAGE_SHIFT) & 0xFF;
        let l2descriptor = &l2_for_phy.descriptors[l2_index];
        if !l2descriptor.is_present() {
            return None;
        }

        let p = ::mem::PhysicalAddress(l2descriptor.get_physical_address().0 | (v.0 & PAGE_MASK));

        Some(p)
    }
}

impl ::mem::MemoryMapper for PageTable {
    fn map(&self,
           fa: &FrameAllocator,
           p: ::mem::PhysicalAddress,
           v: ::mem::VirtualAddress,
           size: MemorySize)
           -> Result<(), ()> {
        let pages = ::mem::to_pages(size).ok().unwrap();
        for i in 0..pages {
            self.cpu_mutex.lock().map_single(fa, p.uoffset(i << PAGE_SHIFT), v.uoffset(i << PAGE_SHIFT));
        }

        Ok(())
    }

    // TODO add 1mb section; to help speed up things up!
    fn map_device(&self,
                  frameallocator: &::mem::FrameAllocator,
                  p: ::mem::PhysicalAddress,
                  v: ::mem::VirtualAddress,
                  size: MemorySize)
                  -> Result<(), ()> {
        let pages = ::mem::to_pages(size).ok().unwrap();
        let bytes = ::mem::to_bytes(size);
        let end_physical = p.uoffset(pages << PAGE_SHIFT);

        let bytes_before_mb = mb_up(p.0) - p.0;
        let bytes_after_mb = end_physical.0 - mb_down(end_physical.0);

        let pages_before_mb = bytes_before_mb >> PAGE_SHIFT;
        let mega_bytes = bytes >> MB_SHIFT;
        let pages_after_mb = bytes_after_mb >> PAGE_SHIFT;

        let mut curr_offset: usize = 0;

        for _ in 0..pages_before_mb {
            self.cpu_mutex.lock().map_single_descriptor(frameallocator,
                                       L2TableDescriptor::new_device(p.uoffset(curr_offset)),
                                       v.uoffset(curr_offset));
            curr_offset += PAGE_SIZE;
        }

        for _ in 0..mega_bytes {
            
            self.cpu_mutex.lock().map_section(L1TableDescriptor::new_section(
                                                p.uoffset(curr_offset),
                                                false), v.uoffset(curr_offset));
            curr_offset += MB_SIZE;
        }

        for _ in 0..pages_after_mb {
            self.cpu_mutex.lock().map_single_descriptor(frameallocator,
                                       L2TableDescriptor::new_device(p.uoffset(curr_offset)),
                                       v.uoffset(curr_offset));
            curr_offset += PAGE_SIZE;
        }

        Ok(())
    }

    fn unmap(&self,
             _: &FrameAllocator,
             _: ::mem::VirtualAddress,
             _: MemorySize)
             -> Result<(), ()> {
        unimplemented!();
    }
}
impl ::mem::PVMapper for PageTable {
    fn p2v(&self, p: ::mem::PhysicalAddress) -> Option<::mem::VirtualAddress> {
        self.cpu_mutex.lock().p2v(p)
    }

    fn v2p(&self, v: ::mem::VirtualAddress) -> Option<::mem::PhysicalAddress> {
        self.cpu_mutex.lock().v2p(v)
    }
}

impl L1Table {
    unsafe fn from_virt_address_no_init(v: ::mem::VirtualAddress) -> L1Table {
        let l1slice: &'static mut [L1TableDescriptor] =
            slice::from_raw_parts_mut(v.0 as *mut L1TableDescriptor, L1TABLE_ENTRIES);
        L1Table { descriptors: l1slice }
    }
    unsafe fn from_virt_address_init(v: ::mem::VirtualAddress) -> L1Table {
        let l1 = Self::from_virt_address_no_init(v);
        for elem in l1.descriptors.iter_mut() {
            *elem = L1TableDescriptor(0);
        }
        l1
    }
        
}


impl L2Table {
    unsafe fn from_virt_address_no_init(v: ::mem::VirtualAddress) -> L2Table {
        let l2slice: &'static mut [L2TableDescriptor] =
            slice::from_raw_parts_mut(v.0 as *mut L2TableDescriptor, L2TABLE_ENTRIES);
        L2Table { descriptors: l2slice }
    }

    unsafe fn from_virt_address_init(v: ::mem::VirtualAddress) -> L2Table {
        let l2 = Self::from_virt_address_no_init(v);
        for elem in l2.descriptors.iter_mut() {
            *elem = L2TableDescriptor(0);
        }
        l2
    }
}
