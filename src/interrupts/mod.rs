use crate::{gdt::DOUBLE_FAULT_IST_IDX, println, serial_println};
use core::arch::naked_asm;
use idt::EntryOptions;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
mod idt;

/* ===== HARDWARE INTERRUPTS ===== */

/*
Programmable Interrupt Controller (PIC)
- Aggregates interrupts from external devices and notifies the CPU
    - Orders these interrupts by priority level
        - e.g. The system timer will be higher priority than the keyboard
- Allows the CPU to skip having to poll all of the connected devices
- Hardware interrupts can occur asynchronously
    - Much faster
    - Much more dangerous
        - Rust's ownership model protects us by forbidding mutable global state
        - Deadlocks are still possible

Simplified PIC setup:
                        ____________             _____
   Timer ------------> |            |           |     |
   Keyboard ---------> | Interrupt  |---------> | CPU |
   Other Hardware ---> | Controller |           |_____|
   Etc. -------------> |____________|

Typical system's 2 PIC setup:
                     ____________                          ____________
Real Time Clock --> |            |   Timer -------------> |            |
ACPI -------------> |            |   Keyboard-----------> |            |      _____
Available --------> | Secondary  |----------------------> | Primary    |     |     |
Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
Primary ATA ------> |            |   Floppy disk -------> |            |
Secondary ATA ----> |____________|   Parallel Port 1----> |____________|

- This allows us to connect many more devices
- Also allows for more levels of priority
- Each controller is configured w/ 2 I/O ports [command, data] at set memory locations
    - Primary Interrupt Controller: [command: 0x20, data: 0x21]
    - Secondary Interrupt Controller: [command: 0xa0, data: 0xa1]
*/

// need to add an offset to the PIC outputs since they typically output in range 0->15
// these values are already taken by the interrupt handlers though so usually the range
// 32->47 is chosen since they're the first free numbers following the 32 exception slots
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// creates a 2 PIC setup illustrated above and locks behind a mutex to allow for safe global accesses
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// define interrupt handler indices
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    SIC,
    Serial2,
    Serial1,
    ParallelPort23,
    Floppy,
    ParallelPort1,
}

// helper functions for quickly changing their data type
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        return self as u8;
    }

    fn as_usize(self) -> usize {
        return self as usize;
    }
}

extern "C" fn timer_interrupt_handler(_stack_frame: &ExceptionStackFrame) {
    // going to leave this blank for now since it's a bit distracting
    crate::print!("");

    // sends explicit End Of Interrupt (EOI) signal to PIC so it can receive the next interrupt
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "C" fn keyboard_interrupt_handler(_stack_frame: &ExceptionStackFrame) {
    use x86_64::instructions::port::Port;

    /*
        Setup a port to read the scancode sent by the keyboard

        Note:
        - The keyboard will not send another scancode until the current one
          is read
        - This handler works for a PS/2 controller keyboard but QEMU will
          emulate that for now
            - The data port for the PS/2 controller is 0x60
    */
    let mut p = Port::new(0x60);

    // read, translate, and display the scancode received
    let scancode: u8 = unsafe { p.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/* ===== IDT TABLE ===== */
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
                naked_asm!("
                    push rax;
                    push rcx;
                    push rdx;
                    push rsi;
                    push rdi;
                    push r8;
                    push r9;
                    push r10;
                    push r11;
                    mov rdi, rsp;
                    add rdi, 9*8;
                    call {};
                    pop r11;
                    pop r10;
                    pop r9;
                    pop r8;
                    pop rdi;
                    pop rsi;
                    pop rdx;
                    pop rcx;
                    pop rax;
                    iretq", sym $name);
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
                naked_asm!("
                    pop rsi;
                    push rax;
                    push rcx;
                    push rdx;
                    push rsi;
                    push rdi;
                    push r8;
                    push r9;
                    push r10;
                    push r11;
                    mov rdi, rsp;
                    add rdi, 9*8;
                    call {}
                    pop r11;
                    pop r10;
                    pop r9;
                    pop r8;
                    pop rdi;
                    pop rsi;
                    pop rdx;
                    pop rcx;
                    pop rax;
                    iretq", sym $name);
            }
        }
        wrapper
    }};
}

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(zero_div_handler), None);
        idt.set_handler(3, handler!(breakpt_handler), None);
        idt.set_handler(6, handler!(invalid_op_handler), None);
        // set double fault handler options (IST index)
        let mut double_fault_options = EntryOptions::new();
        double_fault_options.set_stack_idx(DOUBLE_FAULT_IST_IDX + 1);
        idt.set_handler(8, handler_with_errcode!(double_fault_handler), Some(double_fault_options));
        idt.set_handler(14, handler_with_errcode!(pg_fault_handler), None);
        idt.set_handler(InterruptIndex::Timer.as_usize(), handler!(timer_interrupt_handler), None);
        idt.set_handler(InterruptIndex::Keyboard.as_usize(), handler!(keyboard_interrupt_handler), None);
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
// again align(8) to ensure that the ExceptionStack frame remains aligned on a multiple of 0x8 in memory
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
    crate::hlt_loop();
}

extern "C" fn breakpt_handler(stack_frame: &ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT (INT3)\n{:#x?}", &*stack_frame);
}

extern "C" fn invalid_op_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: INVALID OPCODE\n{:#x?}", &*stack_frame);
    crate::hlt_loop();
}

// disabling this for now until the double-fault handler is finished for testing
#[allow(dead_code)]
extern "C" fn overflow_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("EXCEPTION: OVERFLOW\n{:#x?}", &*stack_frame);
    crate::hlt_loop();
}

extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, err_code: u64) -> ! {
    println!(
        "EXCEPTION: DOUBLE FAULT with error code: {:#x}\n{:#x?}",
        err_code, &*stack_frame
    );
    crate::hlt_loop();
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
    use x86_64::registers::control::Cr2;
    let error = match err_code {
        0x1 => "PROTECTION_VIOLATION",
        0x2 => "CAUSED_BY_WRITE",
        0x4 => "USER_MODE",
        0x8 => "MALFORMED_TABLE",
        0x10 => "INSTRUCTION_FETCH",
        _ => "UNKNOWN",
    };

    /*
        Get the physical address of the top level page table by reading the
        CR3 register, which stores a pointer to said page table in memory

            - x86_64 supports 4-level page tables
                - some support up to 5 levels but they are still compatible
                  with 4-level page tables
    */
    println!(
        "EXCEPTION: PAGE FAULT\nAddr: {:#x}\nError Code: {}\n{:#x?}",
        Cr2::read().as_u64(),
        error,
        &*stack_frame
    );
    crate::hlt_loop();
}

/* ===== INIT ===== */
pub fn init() {
    IDT.load();
}

/* ===== TESTING ===== */

// IDT to be used in integration tests where we can install test handlers
lazy_static! {
    pub static ref TEST_IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(test_zero_div_handler), None);
        idt
    };
}

extern "C" fn test_zero_div_handler(_stack_frame: &ExceptionStackFrame) -> ! {
    serial_println!("[ok]");
    crate::exit_qemu(crate::QEMUExitCode::Success);
    crate::hlt_loop();
}

pub fn init_test() {
    TEST_IDT.load();
}
