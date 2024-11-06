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
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

use core::panic::PanicInfo;
mod vga_buf;

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
