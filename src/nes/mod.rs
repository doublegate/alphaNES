// src/nes/mod.rs
pub mod cpu;
pub mod ppu;

pub struct Nes {
    pub cpu: cpu::Cpu2A03<Bus>,
    pub ppu: ppu::Ppu,
    pub cycles: usize,
}

impl Nes {
    pub fn new(rom: Rom) -> Self {
        let bus = Bus::new(rom);
        let ppu = ppu::Ppu::new(rom.mirroring);
        Self {
            cpu: cpu::Cpu2A03::new(bus),
            ppu,
            cycles: 0,
        }
    }

    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step();
        self.cycles += cpu_cycles as usize;
        
        for _ in 0..cpu_cycles * 3 {
            if self.ppu.step() {
                // Handle frame completion
            }
        }
        
        if self.ppu.nmi_occurred {
            self.cpu.trigger_nmi();
            self.ppu.nmi_occurred = false;
        }
    }
}
// pub mod apu;
// pub mod cart;
