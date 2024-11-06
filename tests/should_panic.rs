#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// defining custom test runner for tests that should panic since #[should_panic] is
// unavailable without the std lib
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_practice::{exit_qemu, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(os_practice::QEMUExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
        serial_println!("[test did not panic]");
        exit_qemu(os_practice::QEMUExitCode::Failure);
    }
    exit_qemu(os_practice::QEMUExitCode::Success);
}

#[test_case]
fn should_fail() {
    use os_practice::serial_print;
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
