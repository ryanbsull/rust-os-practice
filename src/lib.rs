#![no_std]
// condition attribute no_main on if the tests are running
#![cfg_attr(test, no_main)]
#![feature(naked_functions)]
// custom test frameworks requires no external libraries thus works in a #![no_std] environment
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate bit_field;
use core::arch::asm;
use core::panic::PanicInfo;
pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buf;

/* EXCEPTION HANDLER TESTING FUNCTIONS */

// need to create a custom divide by zero function since Rust runtime-checker will catch it otherwise
pub fn divide_by_zero() {
    unsafe { asm!("mov dx, 0", "div dx",) }
}

pub fn invalid_opcode() {
    unsafe { asm!("ud2") }
}

pub fn page_fault() {
    unsafe { *(0xdeadbee8 as *mut u64) = 12 }
}

pub fn breakpoint() {
    x86_64::instructions::interrupts::int3();
}

// keep this function here in case I want to test a stack overflow again
#[allow(unconditional_recursion)]
pub fn overflow() {
    overflow();
}

// create 32-bit exit code enum for QEMU exit port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QEMUExitCode {
    Success = 0x10,
    Failure = 0x11,
}

// track QEMU exit port value, defined in Cargo.toml
const QEMU_PORT: u16 = 0xf4;

pub fn exit_qemu(exit_code: QEMUExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(QEMU_PORT);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QEMUExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QEMUExitCode::Failure);
    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

// move the testing function from main.rs to lib.rs, now the entire function
// _start is only run when testing here
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

// configure a different panic handler to run while testing, otherwise QEMU
// will just hang on the original panic handler plus the output will not be
// visible on the host computer
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

pub fn init() {
    // init the GDT before so the IST is setup for our handlers
    gdt::init();
    interrupts::init();
    // initialize the PICs to handle hardware interrupts
    unsafe { interrupts::PICS.lock().initialize() };
    // enable CPU interrupts
    // executes `sti` ("set interrupts") instruction to enable external interrupts
    x86_64::instructions::interrupts::enable();
}
