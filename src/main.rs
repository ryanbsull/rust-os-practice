// build with `cargo bootimage` to generate the kernel .bin image
// makes sure std lib is not compiled with the program
// *needed if we are to make a free-standing binary for the simple OS
#![no_std]
#![no_main]
// custom test frameworks requires no external libraries thus works in a #![no_std] environment
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QEMUExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

// track QEMU exit port value, defined in Cargo.toml
const QEMU_PORT: u16 = 0xf4;
use core::panic::PanicInfo;
mod vga_buf;

// create 32-bit exit code enum for QEMU exit port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QEMUExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QEMUExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(QEMU_PORT);
        port.write(exit_code as u32);
    }
}
// function called in the event of a panic

/// return type = ! ("never" type) as it will just loop and never return
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}\n", info);
    loop {}
}

/* entry point for the free-standing binary

    requires the "no-mangle" attribute to ensure its name is preserved
    as Rust will change function names to ensure no duplicate names

    'extern "C"' tells the compiler to use the C calling convention
    instead of the Rust calling convention
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", '!');
    #[cfg(test)]
    test_main();
    loop {}
}
