pub struct PpuMemory {
    pub vram: [u8; 2048],
    pub palette: [u8; 32],
    pub oam: [u8; 256],
    pub temp_oam: [u8; 32],
    pub mirroring: Mirroring,
}

#[derive(Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

impl PpuMemory {
    pub fn new(mirroring: Mirroring) -> Self {
        Self {
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
            temp_oam: [0; 32],
            mirroring,
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        let addr = match addr {
            0x2000..=0x3EFF => self.mirror_vram_addr(addr),
            0x3F00..=0x3FFF => self.palette_addr(addr),
            _ => addr,
        };
        self.vram[(addr % 0x4000) as usize]
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let addr = addr - 0x2000;
        match self.mirroring {
            Mirroring::Horizontal => addr & 0x7FF | (addr & 0x800) >> 1,
            Mirroring::Vertical => addr & 0xBFF,
            Mirroring::FourScreen => addr,
        }
    }

    fn palette_addr(&self, addr: u16) -> u16 {
        let addr = addr - 0x3F00;
        if addr == 0x10 || addr == 0x14 || addr == 0x18 || addr == 0x1C {
            addr - 0x10
        } else {
            addr
        }
    }
}
