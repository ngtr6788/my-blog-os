#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(my_blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use my_blog_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
  test_main();

  loop {}
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
  my_blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
  println!("test_println shouldn't fail");
}
