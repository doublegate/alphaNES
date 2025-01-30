use bitflags::bitflags;

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE_X      = 0b00000001;
        const NAMETABLE_Y      = 0b00000010;
        const VRAM_INCREMENT   = 0b00000100;
        const SPRITE_TABLE     = 0b00001000;
        const BACKGROUND_TABLE = 0b00010000;
        const SPRITE_SIZE      = 0b00100000;
        const MASTER_SLAVE     = 0b01000000;
        const NMI_ENABLE       = 0b10000000;
    }
}

bitflags! {
    pub struct MaskRegister: u8 {
        const GRAYSCALE        = 0b00000001;
        const SHOW_BACKGROUND  = 0b00000010;
        const SHOW_SPRITES     = 0b00000100;
        const SHOW_EDGES       = 0b00010000;
        const EMPHASIZE_RED    = 0b00100000;
        const EMPHASIZE_GREEN  = 0b01000000;
        const EMPHASIZE_BLUE   = 0b10000000;
    }
}

#[derive(Default)]
pub struct PpuRegisters {
    pub control: ControlRegister,
    pub mask: MaskRegister,
    pub status: u8,
    pub oam_addr: u8,
    pub scroll: (u8, u8),
    pub addr: u16,
    pub data: u8,
    pub latch: bool,
    pub write_toggle: bool,
}
