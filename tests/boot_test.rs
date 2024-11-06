#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// import test_runner from lib.rs
#![test_runner(os_practice::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_practice::println;

/*
   currently simple but as the kernel grows more complex, having good integration
   testing setups will be key to see how all of the parts will interact going forward
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_practice::test_panic_handler(info);
}

/*
   redundant as already tested by vga_buf module but useful in the future
   when the kernel will call more initialization functions and needs to make
   sure that printing to the screen is viable at boot time
*/
#[test_case]
fn test_println() {
    println!("test_println output");
}
