mod registers;
mod memory;
mod renderer;

use registers::{ControlRegister, MaskRegister, PpuRegisters};
use memory::PpuMemory;
use renderer::PpuRenderer;

pub struct Ppu {
    pub registers: PpuRegisters,
    pub memory: PpuMemory,
    renderer: PpuRenderer,
    pub cycle: usize,
    pub scanline: i16,
    pub frame: u32,
    pub nmi_occurred: bool,
    pub vram_addr: u16,
    pub tram_addr: u16,
    pub fine_x: u8,
}

impl Ppu {
    pub fn new(mirroring: Mirroring) -> Self {
        Self {
            registers: PpuRegisters::default(),
            memory: PpuMemory::new(mirroring),
            renderer: PpuRenderer::new(),
            cycle: 0,
            scanline: -1,
            frame: 0,
            nmi_occurred: false,
            vram_addr: 0,
            tram_addr: 0,
            fine_x: 0,
        }
    }

    pub fn step(&mut self) -> bool {
        let mut frame_complete = false;
        
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            
            if self.scanline > 260 {
                self.scanline = -1;
                self.frame += 1;
                frame_complete = true;
            }
        }
        
        match self.scanline {
            -1 => self.pre_render_scanline(),
            0..=239 => self.visible_scanline(),
            240 => {} // Post-render
            241 => {
                if self.scanline == 241 && self.cycle == 1 {
                    self.registers.status |= 0x80; // VBlank
                    if self.registers.control.contains(ControlRegister::NMI_ENABLE) {
                        self.nmi_occurred = true;
                    }
                }
            },
            _ => {}
        }
        
        frame_complete
    }

    fn pre_render_scanline(&mut self) {
        if self.cycle == 1 {
            self.registers.status &= 0x1F; // Clear VBlank, sprite 0 hit, overflow
        }
    }

    fn visible_scanline(&mut self) {
        if self.cycle < 256 || (self.cycle >= 321 && self.cycle <= 336) {
            self.increment_x();
        }
        
        if self.cycle == 256 {
            self.increment_y();
        }
        
        if self.cycle == 257 {
            self.transfer_x();
        }
        
        if self.scanline == 0 && self.cycle >= 280 && self.cycle <= 304 {
            self.transfer_y();
        }
    }

    fn increment_x(&mut self) {
        if (self.vram_addr & 0x001F) == 31 {
            self.vram_addr &= !0x001F;
            self.vram_addr ^= 0x0400;
        } else {
            self.vram_addr += 1;
        }
    }

    fn increment_y(&mut self) {
        if (self.vram_addr & 0x7000) != 0x7000 {
            self.vram_addr += 0x1000;
        } else {
            self.vram_addr &= !0x7000;
            let mut y = (self.vram_addr & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.vram_addr ^= 0x0800;
            } else {
                y += 1;
            }
            self.vram_addr = (self.vram_addr & !0x03E0) | (y << 5);
        }
    }
}
