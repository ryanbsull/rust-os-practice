#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_practice::{exit_qemu, serial_print, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[panicked but did not interrupt]");
    exit_qemu(os_practice::QEMUExitCode::Failure);
    os_practice::hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    os_practice::interrupts::init_test();
    serial_println!("Running 1 tests:");
    test_zero_division();
    exit_qemu(os_practice::QEMUExitCode::Failure);
    os_practice::hlt_loop();
}

fn test_zero_division() {
    use os_practice::divide_by_zero;
    serial_print!("interrupt_test::test_zero_division...\t");
    divide_by_zero();
}
