use super::println;
use lazy_static::lazy_static;
mod idt;

/*
IDT Table:

TABLE_IDX    |    INTERRUPT_TYPE
----------------------------------
  0x00       |    Division by zero
  0x01       |    Single-step interrupt
  0x02       |    Non-maskable interrupt
  0x03       |    Breakpoint (INT 3)
  0x04       |    Overflow
  0x05       |    Bound Range Exceeded
  0x06       |    Invalid Opcode
  0x07       |    Coprocessor not available
  0x08       |    Double Fault
  0x09       |    Coprocessor Segment Overrun (386 or earlier only)
  0x0A       |    Invalid Task State Segment
  0x0B       |    Segment not present
  0x0C       |    Stack Segment fault
  0x0D       |    General Protection Fault
  0x0E       |    Page Fault
  0x0F       |    *reserved*
  ... IDT for x86 continues but we will only worry about these
*/

// list of all the handler functions
// TODO: make macro to auto populate with all functions in file with "_handler" postfix
const HANDLER_FUNCS: [extern "C" fn() -> !; 6] = [
    zero_div_handler,
    ss_int_handler,
    nmi_handler,
    breakpt_handler,
    overflow_handler,
    bre_handler,
];

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        for (i, handler) in HANDLER_FUNCS.iter().enumerate() {
            idt.set_handler(i as u8, *handler);
        }
        idt
    };
}

extern "C" fn zero_div_handler() -> ! {
    println!("EXCEPTION: DIVSION BY ZERO");
    loop {}
}

extern "C" fn ss_int_handler() -> ! {
    println!("EXCEPTION: SINGLE STEP INTERRUPT");
    /* TODO: IMPLEMENT */
    loop {}
}

extern "C" fn nmi_handler() -> ! {
    println!("EXCEPTION: NON-MASKABLE INTERRUPT");
    /* TODO: IMPLEMENT */
    loop {}
}

extern "C" fn breakpt_handler() -> ! {
    println!("EXCEPTION: BREAKPOINT (INT3)");
    /* TODO: IMPLEMENT */
    loop {}
}

extern "C" fn overflow_handler() -> ! {
    println!("EXCEPTION: OVERFLOW");
    /* TODO: IMPLEMENT */
    loop {}
}

extern "C" fn bre_handler() -> ! {
    println!("EXCEPTION: BOUND RANGE EXCEEDED");
    /* TODO: IMPLEMENT */
    loop {}
}

pub fn init() {
    IDT.load();
}
