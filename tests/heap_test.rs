#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// import test_runner from lib.rs
#![test_runner(os_practice::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kern_main);

fn kern_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;

    os_practice::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { os_practice::mem::init(phys_mem_offset) };
    let mut frame_alloc =
        unsafe { os_practice::mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    os_practice::heap::init_heap(&mut mapper, &mut frame_alloc)
        .expect("Heap initialization failed");

    test_main();
    os_practice::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_practice::test_panic_handler(info)
}

use alloc::boxed::Box;
#[test_case]
fn simple_alloc() {
    let x0 = Box::new(8);
    let x1 = Box::new(22);
    assert_eq!(*x0, 8);
    assert_eq!(*x1, 22);
}

use alloc::vec;
#[test_case]
fn dynamic_vec() {
    let n = 1000;
    let mut v = vec![];
    for i in 0..n {
        v.push(i);
    }
}

use os_practice::heap::HEAP_SIZE;
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
