#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_practice::{exit_qemu, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(os_practice::QEMUExitCode::Success);
    os_practice::hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(os_practice::QEMUExitCode::Failure);
    os_practice::hlt_loop();
}

fn should_fail() {
    use os_practice::serial_print;
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
