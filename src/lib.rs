#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;

pub mod cpu;
pub mod mmu;
mod bits;