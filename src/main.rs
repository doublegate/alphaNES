// src/main.rs
use log::{debug, info, warn};
use nes::cpu::{Bus, Cpu2A03};

const RAM_SIZE: usize = 2048; // 2KB NES RAM

struct NesBus {
    ram: [u8; RAM_SIZE],
    prg_rom: Vec<u8>,      // Cartridge program ROM
    ppu_registers: [u8; 8],// PPU register placeholder
    frame_counter: usize,  // For simulating NMIs
    cycles: usize,         // Global cycle counter
}

impl NesBus {
    fn new() -> Self {
        // Initialize with dummy PRG ROM (test program)
        let mut prg_rom = vec![0; 0x8000];
        
        // Simple test program:
        // Reset handler: LDA #$FF, STA $0000, JMP $8000
        prg_rom[0] = 0xA9; // LDA Immediate
        prg_rom[1] = 0xFF;
        prg_rom[2] = 0x8D; // STA Absolute
        prg_rom[3] = 0x00;
        prg_rom[4] = 0x00;
        prg_rom[5] = 0x4C; // JMP Absolute
        prg_rom[6] = 0x00;
        prg_rom[7] = 0x80;

        Self {
            ram: [0; RAM_SIZE],
            prg_rom,
            ppu_registers: [0; 8],
            frame_counter: 0,
            cycles: 0,
        }
    }

    fn handle_ppu(&mut self, cycles: usize) {
        // Simulate PPU operation (3 PPU cycles per CPU cycle)
        let ppu_cycles = cycles * 3;
        self.frame_counter += ppu_cycles;
        
        // Generate NMI every ~29780 cycles (60Hz frame rate)
        if self.frame_counter >= 29780 {
            self.frame_counter -= 29780;
            // Normally this would trigger the NMI in the CPU
        }
    }

    fn handle_apu(&mut self, cycles: usize) {
        // Simulate APU operation (placeholder)
        let _ = cycles;
    }
}

impl Bus for NesBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM (mirrored every 2KB)
            0x0000..=0x1FFF => {
                let mirrored_addr = addr as usize % 0x0800;
                self.ram[mirrored_addr]
            }
            
            // PPU registers (mirrored every 8 bytes)
            0x2000..=0x3FFF => {
                let reg = (addr - 0x2000) % 8;
                self.ppu_registers[reg as usize]
            }
            
            // APU and I/O registers
            0x4000..=0x4017 => {
                warn!("APU/I/O read from {:04X} not implemented", addr);
                0
            }
            
            // Cartridge space (PRG ROM)
            0x4020..=0xFFFF => {
                let mut effective_addr = addr as usize - 0x4020;
                if effective_addr >= self.prg_rom.len() {
                    effective_addr %= self.prg_rom.len();
                }
                self.prg_rom[effective_addr]
            }
            
            _ => {
                warn!("Unhandled read from {:04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // RAM
            0x0000..=0x1FFF => {
                let mirrored_addr = addr as usize % 0x0800;
                self.ram[mirrored_addr] = data;
            }
            
            // PPU registers
            0x2000..=0x3FFF => {
                let reg = (addr - 0x2000) % 8;
                self.ppu_registers[reg as usize] = data;
                debug!("PPU write {:02X} to {:04X}", data, addr);
            }
            
            // APU and I/O
            0x4000..=0x4017 => {
                debug!("APU/I/O write {:02X} to {:04X}", data, addr);
            }
            
            // Cartridge space
            0x4020..=0xFFFF => {
                warn!("Cartridge write {:02X} to {:04X} ignored", data, addr);
            }
            
            _ => {
                warn!("Unhandled write {:02X} to {:04X}", data, addr);
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("NES emulator starting...");

    let mut bus = NesBus::new();
    
    // Set reset vector to start of PRG ROM
    bus.write(0xFFFC, 0x00);
    bus.write(0xFFFD, 0x80);
    
    let mut cpu = Cpu2A03::new(bus);
    cpu.reset();

    loop {
        // Execute CPU instruction
        let cycles = cpu.step();
        
        // Update global cycle counter
        cpu.bus.cycles += cycles;
        
        // Simulate other components
        cpu.bus.handle_ppu(cycles);
        cpu.bus.handle_apu(cycles);
        
        // Handle periodic NMIs (VBlank simulation)
        if cpu.bus.frame_counter >= 29780 {
            cpu.trigger_nmi();
        }
        
        // Basic execution control
        if cpu.bus.cycles > 100_000 {
            info!("Cycle limit reached, exiting");
            break;
        }
        
        // Example: Print CPU state every 1000 cycles
        if cpu.bus.cycles % 1000 == 0 {
            debug!(
                "Cycles: {} | PC: {:04X} A: {:02X} X: {:02X} Y: {:02X} SP: {:02X}",
                cpu.bus.cycles, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp
            );
        }
    }
}
