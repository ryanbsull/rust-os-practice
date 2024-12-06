use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
// derive traits to enable copy semantics and make it printable + comparable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// makes sure each enum variant will be stored as a u8 (could use 4 bits if Rust had u4 type)
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/*
   ensure ColorCode has exact same data layout as Color (u8) use transparent
   which is only available for structs with single non-zero member
*/
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// use repr(C) to guarantee struct fields are laid out like C structs and guarantees correct ordering
// default Rust ordering does not guarantee struct field order
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// define height and width of 2D VGA buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// create buffer struct to represent VGA buffer in our module
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/*
   create global VGA Writer interface for outside modules to call without
   creating their own Writer instance

   requires lazy_static crate as Rust cannot convert raw pointers to references
   at compile time
*/
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buf: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// create a Writer type abstraction to allow us to more easily write to the
// VGA buffer and track position
pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    // ensure the compiler knows the lifetime of the buffer is for the length
    // of the whole program (kernel) runtime with 'static
    buf: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_pos;

                let color_code = self.color_code;
                self.buf.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // check if printable ASCII or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // outside of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buf.chars[row][col].read();
                self.buf.chars[row - 1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buf.chars[row][col].write(blank);
        }
    }
}

// implement write_str for Write trait for our VGA buffer writer
// so we can now use write!() and writeln!() macros
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
   define print macros for the entire crate so they can interact
   with the VGA buffer through those macros instead of using the
   global interface
*/

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buf::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// use doc(hidden) to hide function from generated documentation
// as it is a private implementation detail
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // ensures that no interrupt can occur while the mutex is locked
    // helps prevent deadlocks
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// test println! runs
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

// test println! runs many times
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

// test that printing is being written to the screen
#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    let s = "test_println_output test string";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln! failed");
        // use enumerate to get both the position in the str: i and the character: c
        for (i, c) in s.chars().enumerate() {
            // check line above in buffer as println! will move the string up a row after printing
            let screen_char = writer.buf.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    })
}
