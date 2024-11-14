// build with `cargo bootimage` to generate the kernel .bin image
// makes sure std lib is not compiled with the program
// *needed if we are to make a free-standing binary for the simple OS
#![no_std]
#![no_main]
// now can call our testing framework and import it from lib.rs
#![feature(custom_test_frameworks)]
#![test_runner(os_practice::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use os_practice::{interrupts, println};

// function called in the event of a panic
/// return type = ! ("never" type) as it will just loop and never return
#[cfg(not(test))] // set this as the panic handler when not testing
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}\n", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // now can just call testing function from lib.rs
    os_practice::test_panic_handler(info);
}
/* entry point for the free-standing binary

    requires the "no-mangle" attribute to ensure its name is preserved
    as Rust will change function names to ensure no duplicate names

    'extern "C"' tells the compiler to use the C calling convention
    instead of the Rust calling convention
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    interrupts::init();
    os_practice::breakpoint();
    println!("Hello World{}", '!');
    #[cfg(test)]
    test_main();
    loop {}
}
