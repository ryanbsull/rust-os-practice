use bit_field::BitField;
use x86_64::instructions::segmentation;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

// IDT is variably sized w/ up to 256 entries, just going to do 16 for now
// the remaining 240 will be treated as non-present by CPU
pub struct Idt([Entry; 16]);

#[derive(Debug, Clone, Copy)]
// ensures compiler keeps field ordering and does not add any padding between fields
// *must be kept in this order (matches the C structure)
#[repr(C, packed)]
pub struct Entry {
    ptr_low: u16,
    gdt_sel: SegmentSelector,
    // merge option bits [32-47] into options field since rust doesn't have u3 or u1 types
    options: EntryOptions,
    ptr_mid: u16,
    ptr_high: u32,
    reserved: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

#[allow(dead_code)]
impl EntryOptions {
    // exclusively handles the requirements
    fn minimal() -> Self {
        let mut options = 0x0000;
        options.set_bits(9..12, 0b111); // bits 9..12 must always be '1'
        EntryOptions(options)
    }

    // sets options to a reasonable preset value
    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    //// All functions return &mut Self to allow for easy method chaining as seen above ^ ////

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_bits(13..15, dpl);
        self
    }

    pub fn set_stack_idx(&mut self, idx: u16) -> &mut Self {
        self.0.set_bits(0..3, idx);
        self
    }
}

/*
    HandlerFunc for our IDT must have:
    1. A defined calling convention as it is being called directly by hardware
        - De-facto in OS dev is "C" so we will use that here
    2. Zero arguments, hardware does not supply arguments to the handler when jumping
    3. A 'never' (!) return type
        - never type aka diverging
        - b/c the hardware does not "call" the handler but jumpt to it after pushing values to the stack,
        this means that the handler function cannot return normally b/c if it did, the system would pop the
        return addr from the stack and it could get a completely different value from then. (e.g The CPU
        pushes an error code onto the stack, if this is interpretted as a return address we could jump to
        invalid memory)
        - also since on an interrupt the register values are overwritten the interrupted function loses
        its state and cannot proceed so returning from the handler like a normal function is useless
*/
pub type HandlerFunc = extern "C" fn() -> !;

// define IDT entry functions
impl Entry {
    fn new(gdt_sel: SegmentSelector, handler: HandlerFunc) -> Self {
        let ptr = handler as u64;
        Entry {
            gdt_sel,
            ptr_low: ptr as u16,
            ptr_mid: (ptr >> 16) as u16,
            ptr_high: (ptr >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }

    // create a non-present IDT entry
    fn missing() -> Self {
        Entry {
            gdt_sel: SegmentSelector::new(0, PrivilegeLevel::Ring0), // Ring0: kernel level privilege
            ptr_low: 0,
            ptr_mid: 0,
            ptr_high: 0,
            options: EntryOptions::minimal(), // only set the must-be-one bits for a missing IDT entry
            reserved: 0,
        }
    }
}

impl Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 16])
    }

    // from phil-opp.com: originally returned &mut EntryOptions but cannot return unaligned field now
    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) {
        self.0[entry as usize] = Entry::new(segmentation::cs(), handler);
    }

    // IDT must be valid until a new IDT is loaded and as long as the kernel runs, thus "'static"
    // this will ensure that the IDT is not overwritten since we initially construct it on the stack
    // before loading. If it was not static then a function call could overwrite the IDT memory location
    // and lead to unknown code being executed in the event of an interrupt
    pub fn load(&'static self) {
        use core::mem::size_of;
        use x86_64::instructions::tables::{lidt, DescriptorTablePointer};
        use x86_64::VirtAddr;

        let ptr = DescriptorTablePointer {
            base: VirtAddr::from_ptr(self as *const _),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}
