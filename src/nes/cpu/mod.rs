// src/nes/cpu/mod.rs
// CPU module
mod ricoh_2a03_cpu;

// Re-export public interface
pub use ricoh_2a03_cpu::{Bus, Cpu2A03, InterruptType};