pub trait Bus {
    // Bus trait methods
}

pub struct Cpu2A03 {
    // CPU struct fields and methods
}

impl Cpu2A03 {
    pub fn new(bus: impl Bus) -> Self {
        Cpu2A03 {
            // Initialize fields here
        }
    }

    pub fn reset(&mut self) {
        // Implementation
    }

    pub fn step(&mut self) -> usize {
        // Example implementation returning a usize value
        0
    }

    pub fn trigger_nmi(&mut self) {
        // Implementation
    }
}