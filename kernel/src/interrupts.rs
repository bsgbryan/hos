use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}