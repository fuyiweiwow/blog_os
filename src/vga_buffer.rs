use volatile::Volatile;
use core::fmt;

#[allow(dead_code)] //SyntaxTip: Avoid warning of this piece of code if it is unused
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //SyntaxTip: Make the enum printable and comparable
#[repr(u8)]//SyntaxTip:  Indicate each enum variant is stored as u8, u4 is enough, but Rust has no u4
pub enum Color {
    Black       = 0,
    Blue        = 1,    
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Magenta     = 5,
    Brown       = 6, 
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    Pink        = 13,
    Yellow      = 14,
    White       = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] //SyntaxTip: Indicate the struct has same memory layout as u8 and tell this is a ColorCode not a normal u8
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/*
    Screen character
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] //SyntaxTip: File ordering of default structs is undefined in Rust, this attributre guarantees the stuct's fields behaves in C and thus guarantees the correct ordering
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct  Buffer {
    //SyntaxTip: Volatile guarantees that the reads/writes are not optimized away
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/*
    Write to the last line and shift lines up when it is full(or on \n) of VGA buffer
 */
pub struct  Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, //SyntaxTip: A program life time('static), changeable reference(&mut) of VGA buffer but not ownership(just borrow)
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // for unprintable bytes, print a ■ instead
            }
        }
    }

    fn new_line(&mut self) { 
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;        
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use spin::Mutex; // OS free mutex
use lazy_static::lazy_static;
/*
    Use lazy_static instead of computing its value at compile time,  
    the static lazily initializes itself when accessed for the first time
 */
lazy_static! {
    /*
        Provide a global writer for other modules rather than carry a Writer instance everwhere
        SyntaxTip 1:
            first we cast the integer 0xb8000 as a mutable raw pointer, this step is a standard
            then wr convert it to a mutable reference by dereferencing it (use *) 
            and immediately borrowing it again (use &mut)

            ┌─────────────────┐ 0x00000000
            │                 │
            │                 │
            ├─────────────────┤ 0x000B8000  ←─── 0xb8000
            │  [VGA Display]  │     │
            │  [[ScreenChar;80];25] │      │ as *mut Buffer
            │                 │      ↓
            └─────────────────┘ 0x000B8FA0
                │
                │ *(ptr) deref
                ↓
            ┌─────────────────┐
            │ Actual Buffer   │ ←─── &mut borrow
            └─────────────────┘
    */
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) } //SyntaxTip 1
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga_buffer::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap(); //SyntaxTip: The additional unwrap() at the end panics if printing isn’t successful
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate()  {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}