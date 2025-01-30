pub struct PpuRenderer {
    pub front_buffer: Vec<u32>,
    back_buffer: Vec<u32>,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,
    pub scanline_sprites: Vec<Sprite>,
}

#[derive(Clone)]
struct Sprite {
    y: u8,
    tile: u8,
    attributes: u8,
    x: u8,
    data_low: u8,
    data_high: u8,
}

impl PpuRenderer {
    pub fn new() -> Self {
        Self {
            front_buffer: vec![0; 256 * 240],
            back_buffer: vec![0; 256 * 240],
            sprite_zero_hit: false,
            sprite_overflow: false,
            scanline_sprites: Vec::with_capacity(8),
        }
    }

    pub fn render_scanline(&mut self, ppu: &mut Ppu, scanline: i16) {
        if scanline < 0 || scanline > 239 { return; }

        // Background rendering
        if ppu.registers.mask.contains(MaskRegister::SHOW_BACKGROUND) {
            self.render_background(ppu, scanline);
        }

        // Sprite rendering
        if ppu.registers.mask.contains(MaskRegister::SHOW_SPRITES) {
            self.evaluate_sprites(ppu, scanline);
            self.render_sprites(ppu, scanline);
        }

        // Swap buffers at end of frame
        if scanline == 240 {
            std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        }
    }

    fn render_background(&mut self, ppu: &mut Ppu, scanline: i16) {
        let fine_y = ((ppu.vram_addr >> 12) & 0x7) as u8;
        let coarse_y = ((ppu.vram_addr >> 5) & 0x1F) as u8;
        let nametable = ((ppu.vram_addr >> 10) & 0x3) as u8;

        for x in 0..256 {
            let coarse_x = (ppu.vram_addr & 0x1F) as u8;
            let tile = ppu.memory.read_vram(0x2000 | (ppu.vram_addr & 0xFFF));
            
            // Fetch pattern data
            let pattern_addr = ppu.registers.control.bits() << 12 
                | (tile as u16) << 4 
                | fine_y as u16;
            
            let pattern_low = ppu.memory.read_vram(pattern_addr);
            let pattern_high = ppu.memory.read_vram(pattern_addr + 8);
            
            // Get palette
            let attr_addr = 0x23C0 | (ppu.vram_addr & 0xC00) 
                | ((ppu.vram_addr >> 4) & 0x38) 
                | ((ppu.vram_addr >> 2) & 0x07);
            let attr = ppu.memory.read_vram(attr_addr);
            
            // Calculate pixel color
            let shift = 7 - (x % 8);
            let palette = self.get_background_palette(ppu, attr, coarse_x, coarse_y);
            let color = self.get_color(ppu, palette, pattern_low, pattern_high, shift);
            
            self.back_buffer[(scanline as usize * 256) + x as usize] = color;
        }
    }

    fn evaluate_sprites(&mut self, ppu: &mut Ppu, scanline: i16) {
        self.scanline_sprites.clear();
        let sprite_height = if ppu.registers.control.contains(ControlRegister::SPRITE_SIZE) {
            16
        } else {
            8
        };

        for sprite in (0..64).map(|i| &ppu.memory.oam[i*4..i*4+4]) {
            let y = sprite[0] as i16 + 1;
            if scanline >= y && scanline < y + sprite_height {
                if self.scanline_sprites.len() < 8 {
                    self.scanline_sprites.push(Sprite {
                        y: sprite[0],
                        tile: sprite[1],
                        attributes: sprite[2],
                        x: sprite[3],
                        data_low: 0,
                        data_high: 0,
                    });
                } else {
                    ppu.registers.status |= 0x20; // Sprite overflow
                    break;
                }
            }
        }
    }
}
