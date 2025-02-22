// build with `cargo bootimage` to generate the kernel .bin image
// makes sure std lib is not compiled with the program
// *needed if we are to make a free-standing binary for the simple OS
#![no_std]
#![no_main]
// now can call our testing framework and import it from lib.rs
#![feature(custom_test_frameworks)]
#![test_runner(os_practice::test_runner)]
#![reexport_test_harness_main = "test_main"]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os_practice::{println, task::{keyboard, exec::Exec, Task}};
use x86_64::VirtAddr;

// function called in the event of a panic
/// return type = ! ("never" type) as it will just loop and never return
#[cfg(not(test))] // set this as the panic handler when not testing
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}\n", info);
    os_practice::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // now can just call testing function from lib.rs
    os_practice::test_panic_handler(info);
}

/*
    entry point for the free-standing binary

    using the entry_point!() macro specifies to the bootloader that
    this function is our kernel's entry so extern "C" and the name
    _start() with the #[no_mangle] attribute are no longer needed,
    nor does it need to be a public function

    adding in: boot_info: &'static BootInfo allows it to accept the
    boot information passed by the bootloader

    BootInfo:

        structure passed by the bootloader to the kernel that specifies:
            - memory_map
                - overview of available physical memory
                    - How much physical memory is available
                    - What spaces are reserved for devices e.g. VGA hardware
                - can be queried from BIOS or UEFI firmware but only early
                  in the boot process
                      - that's why it's provided by the bootloader
            - physical_memory_offset
                - virtual address start of the physical memory mapping
                    - add offset to a physical address to get its virtual
                      address
                - Can be customized by adding [package.metadata.bootloader]
                  to Cargo.toml and setting the field physical-memory-offset
                      - e.g physical-memory-offset = "0x0000f00000000000"
                      - Note: bootloader can panic if it runs into physical
                        addresses that overlap w/ the space beyond the
                        offset (areas it would've mapped to some other
                        early physical addresses)
                            - In general the higher the value the better
*/
entry_point!(kern_main);

fn kern_main(boot_info: &'static BootInfo) -> ! {
    os_practice::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { os_practice::mem::init(phys_mem_offset) };
    let mut frame_alloc =
        unsafe { os_practice::mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    os_practice::heap::init_heap(&mut mapper, &mut frame_alloc)
        .expect("Heap initialization failed");

    println!("Hello Kernel!");

    let mut exec = Exec::new();
    exec.spawn(Task::new(example_task()));
    exec.spawn(Task::new(keyboard::print_keypresses()));
    exec.run();

    #[cfg(test)]
    test_main();
    os_practice::hlt_loop();
}

async fn async_num() -> u32 {
    27
}

async fn example_task() {
    let number = async_num().await;
    println!("async num: {}", number);
}
