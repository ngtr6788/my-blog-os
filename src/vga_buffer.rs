// Used to print to the blog OS terminal

use volatile::Volatile;
use x86_64::instructions::interrupts::without_interrupts;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

// NB: A couple of notes on why we use:
// - lazy_static: used to define "dynamic" or lazily defined static variables 
// (we don't need to use actual const functions or actual const/static values)
// - Mutex: we want to have access this synchronously and mutate this,
// because having a mutable static is really dangerous. Using a Mutex 
// spinlock, we get interior mutability and keep data races away
lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    row_position: 0,
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
}

// Print macros, so that print! and println! are used to print to the VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Colours
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
  fn new(foreground: Color, background: Color) -> Self {
    Self((background as u8) << 4 | (foreground as u8))
  }
}

// Screen character representation in VGA buffer
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
struct ScreenChar {
  ascii_char: u8,
  color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

// The actual VGA buffer for the screen
#[repr(transparent)]
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

// The type and methods used to implement the WRITER static variable
pub struct Writer {
  row_position: usize,
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer
}

impl Writer {
  fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => self.new_line(),
      byte => {
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = self.row_position;
        let col = self.column_position;

        self.buffer.chars[row][col].write(ScreenChar {
          ascii_char: byte,
          color_code: self.color_code,
        });

        self.column_position += 1;
      }
    }
  }

  fn write_string(&mut self, string: &str) {
    for byte in string.bytes() {
      match byte {
        // printable ASCII byte or newline
        0x20..=0x7e | b'\n' => self.write_byte(byte),
        // not part of printable ASCII range
        _ => self.write_byte(0xfe)
      }
    }
  }

  fn new_line(&mut self) {
    if self.row_position < BUFFER_HEIGHT - 1 {
      self.row_position += 1;
    } else {
      for row in 1..BUFFER_HEIGHT {
        for col in 0..BUFFER_WIDTH {
          let character = self.buffer.chars[row][col].read();
          self.buffer.chars[row - 1][col].write(character);
        }
      }
      self.clear_row(BUFFER_HEIGHT - 1);
    }
    self.column_position = 0;
  }

  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_char: b' ',
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {

  without_interrupts(|| {
    WRITER.lock().write_fmt(args).unwrap();
  });
}

#[test_case]
fn test_println_simple() {
  println!("This does not panic");
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
  without_interrupts(|| {
    let mut writer = WRITER.lock();
    writeln!(writer, "\n{}", s).expect("writeln failed");
    for (i, c) in s.chars().enumerate() {
      let screen_char = writer.buffer.chars[writer.row_position - 1][i].read();
      assert_eq!(char::from(screen_char.ascii_char), c);
    }
  })
}
