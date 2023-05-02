// Sets up exception and interrupt handlers

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{print, println, gdt};
use pic8259::ChainedPics;
use spin::Mutex;
use pc_keyboard::{Keyboard, layouts::Us104Key, ScancodeSet1, HandleControl, DecodedKey};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  Timer = PIC_1_OFFSET,
  Keyboard,
}

impl InterruptIndex {
  fn as_u8(self) -> u8 {
    self as u8
  }

  fn as_usize(self) -> usize {
    self as usize
  }
}

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_hanlder);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    unsafe {
      idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

lazy_static! {
  static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> = 
    Mutex::new(
      Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
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

extern "x86-interrupt" fn timer_interrupt_hanlder(_stackframe: InterruptStackFrame) {
  print!(".");
  unsafe {
    PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  };
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stackframe: InterruptStackFrame) {
  use x86_64::instructions::port::Port;
  
  let mut ps2_data_port = Port::new(0x60);
  let mut keyboard = KEYBOARD.lock();
  let scancode: u8 = unsafe { ps2_data_port.read() };

  if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    if let Some(key) = keyboard.process_keyevent(key_event) {
      match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key)
      }
    }
  }

  unsafe {
    PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  };
}

#[test_case]
fn test_breakpoint_exception() {
  x86_64::instructions::interrupts::int3();
}
