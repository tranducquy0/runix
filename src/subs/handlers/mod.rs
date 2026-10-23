//! CPU/runtime event handlers: the panic handler, IDT/interrupt dispatch,
//! and the double-fault task.

pub mod doublefault;
pub mod idt;
pub mod panic;
