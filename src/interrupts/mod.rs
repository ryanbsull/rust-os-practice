use crate::println;
use core::arch::naked_asm;
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

// creates a wrapper function to be passed to our set_handler() Idt method
// takes a function identifier $name (not a string of the name nor ptr to function location!)
macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                naked_asm!("mov rdi, rsp; sub rsp, 8; call {}", sym $name);
            }
        }
        wrapper
    }};
}

// the same as above but this time it moves the error code into rsi (the
// second function argument register)
macro_rules! handler_with_errcode {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                naked_asm!("pop rsi; mov rdi, rsp; sub rsp, 8; call {}", sym $name);
            }
        }
        wrapper
    }};
}

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(zero_div_handler));
        idt.set_handler(3, handler!(breakpt_handler));
        idt.set_handler(6, handler!(invalid_op_handler));
        idt.set_handler(14, handler_with_errcode!(pg_fault_handler));
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

// since we now need to call from a naked handler function (which only allows for assembly)
// we need to know the real name of our function since naked_asm prohibits "in(reg)"
extern "C" fn zero_div_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: DIVSION BY ZERO\n{:#x?}", &*stack_frame);
    loop {}
}

extern "C" fn breakpt_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: BREAKPOINT (INT3)\n{:#x?}", &*stack_frame);
    loop {}
}

extern "C" fn invalid_op_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: INVALID OPCODE\n{:#x?}", &*stack_frame);
    loop {}
}

/*
   Page Fault Error Codes:

   PROTECTION_VIOLATION = 1 << 0;
   CAUSED_BY_WRITE = 1 << 1;
   USER_MODE = 1 << 2;
   MALFORMED_TABLE = 1 << 3;
   INSTRUCTION_FETCH = 1 << 4;
*/
extern "C" fn pg_fault_handler(stack_frame: &ExceptionStackFrame, err_code: u64) -> ! {
    let error = match err_code {
        0x1 => "PROTECTION_VIOLATION",
        0x2 => "CAUSED_BY_WRITE",
        0x4 => "USER_MODE",
        0x8 => "MALFORMED_TABLE",
        0x10 => "INSTRUCTION_FETCH",
        _ => "UNKNOWN",
    };
    println!(
        "EXCEPTION: PAGE FAULT with error code: {}\n{:#x?}",
        error, &*stack_frame
    );
    loop {}
}

pub fn init() {
    IDT.load();
}
