#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(my_blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![deny(clippy::all, clippy::pedantic)]

use core::panic::PanicInfo;
use my_blog_os::println;

// Entry point for the OS
#[no_mangle]
pub extern "C" fn _start() -> ! {
  println!("Hello world!");

  my_blog_os::init();

  #[cfg(test)]
  test_main();

  println!("It did not crash!");
  loop {}
}

// Panic implementations
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  my_blog_os::test_panic_handler(info);
}
