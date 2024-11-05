// makes sure std lib is not compiled with the program
// *needed if we are to make a free-standing binary for the simple OS
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// function called in the event of a panic

/// return type = ! ("never" type) as it will just loop and never return
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// entry point for the free-standing binary

/// requires the "no-mangle" attribute to ensure its name is preserved
/// as Rust will change function names to ensure no duplicate names

/// 'extern "C"' tells the compiler to use the C calling convention
/// instead of the Rust calling convention

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
