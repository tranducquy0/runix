// Thin bridge: paste the kernel allocator operators (new/alloc/free/
// counters) into this module so their `super::reusables` references
// resolve against the shim tree, exactly like the kernel.
include!("../../../../src/subs/mem/operators.rs");
