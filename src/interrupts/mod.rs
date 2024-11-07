use super::println;
use lazy_static::lazy_static;
mod idt;

lazy_static! {
    pub static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, divide_by_zero_handler);
        idt
    };
}

extern "C" fn divide_by_zero_handler() -> ! {
    println!("EXCEPTION: DIVSION BY ZERO");
    loop {}
}

pub fn init() {
    IDT.load();
}
