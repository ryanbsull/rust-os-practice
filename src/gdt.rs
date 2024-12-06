use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_IDX: u16 = 0;

/*

Task State Segment (TSS) 64-bit format:

  Field	                  Type
------------------ | ------------
(reserved)	               u32
Privilege Stack Table	[u64; 3]  // used for user-mode programs (ignore for now)
(reserved)	               u64
Interrupt Stack Table	[u64; 7] // see below
(reserved)	               u64
(reserved)	               u16
I/O Map Base Address	   u16

Interrupt Stack Table (IST): (in pseudocode)

struct InterruptStackTable {
    stack_pointers: [Option<StackPointer>; 7],
}

Whenever an exception handler is called we can choose a stack
from the IST through the StackPointer field in the corresponding
IDT entry

e.g. double_fault_handler() could use the first stack in the IST
- The CPU would switch to that stack whenever a double fault occurs
- This switch would happen before anything is pushed
    - prevents a triple fault

*/

// initialize the TSS
// use lazy_static! again to allow for one time static assignment at runtime
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // note: this double_fault_handler() stack as no guard page so if we do
        // anything that uses the stack too much it could overflow and corrupt
        // memory below it
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_IDX as usize] = {
            // calculate size of the stack
            const STACK_SIZE: usize = 4096 * 5;
            // initialize stack memory to all zeroes
            // currently don't have any memory management so need to use `static mut`
            // must be `static mut` otherwise the compiler will map the memory to a
            // read-only page
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // calculate beginning and end of the stack and return a pointer
            // to the end limit of the stack
            #[allow(static_mut_refs)]
            let stack_start = VirtAddr::from_ptr(unsafe {core::ptr::from_ref(&STACK)} );
            stack_start + STACK_SIZE // top of the stack from where it can grow downward
        };
        tss
    };
}

/*
  Global Descriptor Table (GDT)

  - used for memory segmentation before paging became standard
  - still needed in 64-bit mode for stuff like kernel/user mode configuration or TSS loading
  - GDT is a structure that contains segments of the program
      - originally to isolate programs before paging
      - segmentation is no longer supported in 64-bit mode
*/

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // initialize the code segment of the GDT for the kernel and capture the SegmentSelector for it
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // initialize the TSS segment of the GDT and capture the SegmentSelector for it
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {code_selector, tss_selector})
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // reload the code segment register
        CS::set_reg(GDT.1.code_selector);
        // load the TSS
        load_tss(GDT.1.tss_selector);
    }
}
