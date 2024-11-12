use crate::{println, serial_println};
use core::arch::{asm, naked_asm};
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
    zero_div_wrapper,
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

// super hack-y way of doing it but will work for the time being until I understand
// testing harnesses better
lazy_static! {
    pub static ref TEST_IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, zero_div_test_handler);
        idt
    };
}

/*
Exception Stack Frame:

-------- <-- Old stack ptr
Stack Alignment var
--------
Stack Segment (ss)
--------
Return Stack Pointer (rsp)
--------
RFLAGS (8-byte)
--------
Code Segment (cs)
--------
Return Instruction Pointer (rip)
--------
Error Code (optional)
-------- <-- New stack ptr
Handler Function
Stack frame

*/

#[derive(Debug, Default)]
// Note: #[repr(C)] guarantees field order is as stated, Rust representation doesn't!!
#[repr(C, packed)]
// the fields are ordered in reverse since the stack grows downward
struct ExceptionStackFrame {
    instr_ptr: u64,
    code_seg: u64,
    rflags: u64,
    stack_ptr: u64,
    stack_seg: u64,
}

#[naked]
extern "C" fn zero_div_wrapper() -> ! {
    // move rsp -> rdi (rdi is an argument register so essentially it passes
    // the stack pointer as an arg to zero_div_handler)
    unsafe {
        naked_asm!("mov rdi, rsp; call zero_div_handler");
    }
}

// since we now need to call from a naked handler function (which only allows for assembly)
// we need to know the real name of our function since naked_asm prohibits "in(reg)"
#[no_mangle]
extern "C" fn zero_div_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: DIVSION BY ZERO\n{:#x?}", unsafe {
        &*stack_frame
    });
    loop {}
}

// very hack-y but for now will do for testing functionality
extern "C" fn zero_div_test_handler() -> ! {
    serial_println!("[ok]");
    super::exit_qemu(crate::QEMUExitCode::Success);
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

pub fn init_test() {
    TEST_IDT.load();
}
