#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![deny(clippy::all, clippy::pedantic)]

use core::panic::PanicInfo;
mod vga_buffer;
mod serial;

// Entry point for the OS
#[no_mangle]
pub extern "C" fn _start() -> ! {
  println!("Hello world!");

  #[cfg(test)]
  test_main();

  loop {}
}

// QEMU exit codes
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum QemuExitCode {
  Success = 0x10,
  Failure = 0x11
}

pub fn exit_qemu(exit_code: QemuExitCode) {
  use x86_64::instructions::port::Port;

  unsafe {
    let mut port = Port::new(0xf4);
    port.write(exit_code as u32);
  }
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
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failure);
  loop {}
}

// Setting up how tests are run and printed out
pub trait Testable {
  fn run(&self) -> ();
}

impl<T> Testable for T
where 
  T: Fn(),
{
  fn run(&self) {
    serial_print!("{}...\t", core::any::type_name::<T>());
    self();
    serial_println!("[ok]");
  }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} test{}", tests.len(), if tests.len() != 1 { "s" } else { "" });
  for test in tests {
    test.run();
  }
  exit_qemu(QemuExitCode::Success);
}
