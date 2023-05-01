#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

use core::panic::PanicInfo;

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

pub fn init() {
  gdt::init();
  interrupts::init_idt();
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

pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} test{}", tests.len(), if tests.len() != 1 { "s" } else { "" });
  for test in tests {
    test.run();
  }
  exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failure);
  loop {}
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info);
}

// Entry point for the test 
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
  init();
  test_main();

  loop {}
}
