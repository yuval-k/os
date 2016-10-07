pub mod serial;
pub mod pic;
pub mod timer;
pub mod stub;

use core::ops;
use super::mem;
use super::vector;
use super::cpu;

use mem::MemoryMapper;

use device::serial::SerialMMIO;

fn up(a : usize) -> ::mem::PhysicalAddress {::mem::PhysicalAddress((a + mem::PAGE_MASK) & (!mem::PAGE_MASK))}
fn down(a : usize) -> ::mem::PhysicalAddress { ::mem::PhysicalAddress((a ) & (!mem::PAGE_MASK))}

#[no_mangle]
pub extern "C" fn integrator_main(
    sp_end_virt : usize, sp_end_phy : usize, 
    kernel_start_phy : usize, kernel_start_virt : usize, kernel_end_virt : usize,
    l1table_id : usize, l2table_space_id : usize) -> !{

    let ml = mem::MemLayout {
        kernel_start_phy : ::mem::PhysicalAddress(kernel_start_phy),
        kernel_start_virt : ::mem::VirtualAddress(kernel_start_virt),
        kernel_end_virt : ::mem::VirtualAddress(kernel_end_virt),
        stack_phy  : ::mem::PhysicalAddress(sp_end_phy - mem::PAGE_SIZE), // sp points to begining of stack..
        stack_virt :  ::mem::VirtualAddress(sp_end_virt- mem::PAGE_SIZE),
        };

    let skipRanges = [down(kernel_start_virt)..up(kernel_end_virt), down(ml.stack_virt.0) .. up(sp_end_virt), down(l1table_id) .. up(l2table_space_id + 4*mem::L2TABLE_ENTRIES) ];
    // can't use short syntax: https://github.com/rust-lang/rust/pull/21846#issuecomment-110526401
    let mut freedRanges : [Option<ops::Range<::mem::PhysicalAddress>>;10] = [None,None,None,None,None,None,None,None,None,None];
    
    let mut frameAllocator = mem::LameFrameAllocator::new(&skipRanges, &mut freedRanges, 1<<27);

    let mut pageTable = mem::init_page_table(::mem::VirtualAddress(l1table_id), ::mem::VirtualAddress(l2table_space_id), &ml , &mut frameAllocator);

    // init interrupt vectors
    build_mode_stacks(& mut pageTable, &mut frameAllocator);
    // map vector tables
    pageTable.map(&mut frameAllocator, ::mem::PhysicalAddress(0), vector::VECTORS_ADDR, 1);
    vector::build_vector_table();

    // map serial
    pageTable.map_device(&mut frameAllocator, serial::SERIAL_BASE_PADDR, serial::SERIAL_BASE_VADDR);
    // print to serial should work now!

    // map interrupt controller
    pageTable.map_device(&mut frameAllocator, pic::PIC_BASE_PADDR, pic::PIC_BASE_VADDR);
    // map timer
    pageTable.map_device(&mut frameAllocator, timer::TIMER_BASE_PADDR, timer::TIMER_BASE_VADDR);

    let mut w = &mut serial::Writer::new();
    w.writeln("Welcome home!");

    // register irq handler
    vector::get_vec_table().register_irq(interrupt_happened);

    // enable interrupts from the PIC
    cpu::enable_interrupts();
    // enable timer0 interrupt
    pic::enable_interrupts(pic::TIMERINT0);
    // start timer0
    timer::start_timer0();

    ::arch::arm::arm_main(&mut pageTable, frameAllocator);

    loop {}
}

fn interrupt_happened(ctx : & vector::Context) -> Option<vector::Context> {
    if pic::interrupt_status().contains(pic::TIMERINT0) {
        return timer_happened(ctx);
    }
    
    None
}

fn timer_happened(ctx : & vector::Context) -> Option<vector::Context> {
    // clear the interrupt
    timer::clear_interrupt0();

    let mut w = &mut serial::Writer::new();
    w.writeln("timer!!");
    // TODO call scheduler
    
    None
}

fn build_mode_stacks<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : &mut T) {

    const stacks_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xb000_0000);
    // allocate 5 pages
    let pa = frameAllocator.allocate(5).unwrap();

    // 4k per stack; so need 5*4kb memory = five pages
    mapper.map(frameAllocator, pa, stacks_base, 5*mem::PAGE_SIZE);
    
    cpu::set_stack_for_modes(stacks_base);
}
