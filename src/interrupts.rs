// Sets up exception and interrupt handlers

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, gdt};

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
      idt.double_fault.set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

pub fn init_idt() {
  IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stackframe: InterruptStackFrame) {
  println!("EXCEPTION: BREAKPOINT\n{:#?}", stackframe);
}

extern "x86-interrupt" fn double_fault_handler(stackframe: InterruptStackFrame, _error_code: u64) -> ! {
  panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", stackframe);
}

#[test_case]
fn test_breakpoint_exception() {
  x86_64::instructions::interrupts::int3();
}
