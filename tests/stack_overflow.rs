#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(my_blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use my_blog_os::{gdt, serial_println, exit_qemu, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[no_mangle]
pub extern "C" fn _start() -> ! {
  gdt::init(); 
  init_test_idt();
  test_main();

  loop {}
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
  my_blog_os::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
  stack_overflow();
  volatile::Volatile::new(0).read(); // prevents tail recursion optimizations
}

#[test_case]
fn test_stack_overflow() {
  stack_overflow();
}

lazy_static! {
  static ref TEST_IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      idt.double_fault
        .set_handler_fn(test_double_fault_handler)
        .set_stack_index(my_blog_os::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

pub fn init_test_idt() {
  TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stackframe: InterruptStackFrame, _error_code: u64) -> ! {
  serial_println!("[ok]");
  exit_qemu(QemuExitCode::Success);
  loop {}
}
