#![no_std]
#![no_main]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello world!";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
  // Pointer to VGA buffer
  let vga_buffer = 0xb8000 as *mut u8;

  // Iterate over Hello world!
  for (i, &byte) in HELLO.iter().enumerate() {
    unsafe {
      // Think in C: *(vga_buffer + 2 * i) or *(vga_buffer + 2 * i + 1)
      *vga_buffer.offset(2 * i as isize) = byte; // character
      *vga_buffer.offset(2 * i as isize + 1) = 0xb; // cyan colour
    }
  }
  loop {}
}
