// ricoh_2a03_cpu.rs
// Ricoh 2A03/2A07 CPU (NES) emulation core

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(PartialEq)]
enum InterruptType {
    Nmi,
    Irq,
    Brk,
}

const CARRY: u8 = 1 << 0;
const ZERO: u8 = 1 << 1;
const INTERRUPT_DISABLE: u8 = 1 << 2;
const DECIMAL: u8 = 1 << 3;
const BREAK: u8 = 1 << 4;
const OVERFLOW: u8 = 1 << 6;
const NEGATIVE: u8 = 1 << 7;

pub struct Cpu2A03<B: Bus> {
    // Registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: u8,
    
    // Interrupt state
    pub nmi_pending: bool,
    pub irq_pending: bool,
    pub interrupt_mask_delay: bool,
    
    // Memory bus
    pub bus: B,
    
    // Cycle counting
    cycles: usize,
}

impl<B: Bus> Cpu2A03<B> {
    pub fn new(bus: B) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,
            status: 0x34,
            nmi_pending: false,
            irq_pending: false,
            interrupt_mask_delay: false,
            bus,
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.read_u16(0xFFFC);
        self.sp = 0xFD;
        self.status = 0x34;
        self.cycles += 8;
    }

    // Memory operations
    fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.bus.read(addr) as u16;
        let hi = self.bus.read(addr + 1) as u16;
        (hi << 8) | lo
    }

    // Stack operations
    fn push(&mut self, data: u8) {
        self.bus.write(0x0100 | self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read(0x0100 | self.sp as u16)
    }

    // Flag operations
    fn set_flag(&mut self, flag: u8, condition: bool) {
        if condition {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) != 0
    }

    // Addressing modes
    fn imm(&mut self) -> u8 {
        let val = self.bus.read(self.pc);
        self.pc += 1;
        val
    }

    fn zpg(&mut self) -> u16 {
        self.bus.read(self.pc) as u16
    }

    fn abs(&mut self) -> u16 {
        let lo = self.bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.bus.read(self.pc) as u16;
        self.pc += 1;
        (hi << 8) | lo
    }

    fn abs_x(&mut self) -> (u16, bool) {
        let base = self.abs();
        let addr = base.wrapping_add(self.x as u16);
        (addr, (base & 0xFF00) != (addr & 0xFF00))
    }

    fn abs_y(&mut self) -> (u16, bool) {
        let base = self.abs();
        let addr = base.wrapping_add(self.y as u16);
        (addr, (base & 0xFF00) != (addr & 0xFF00))
    }

    fn zpg_x(&mut self) -> u16 {
        (self.zpg() + self.x as u16) & 0xFF
    }

    fn zpg_y(&mut self) -> u16 {
        (self.zpg() + self.y as u16) & 0xFF
    }

    fn idx_ind(&mut self) -> u16 {
        let ptr = (self.zpg() + self.x as u16) & 0xFF;
        let lo = self.bus.read(ptr) as u16;
        let hi = self.bus.read((ptr + 1) & 0xFF) as u16;
        (hi << 8) | lo
    }

    fn ind_idx(&mut self) -> (u16, bool) {
        let base = self.zpg();
        let lo = self.bus.read(base) as u16;
        let hi = self.bus.read((base + 1) & 0xFF) as u16;
        let effective = (hi << 8) | lo;
        let addr = effective.wrapping_add(self.y as u16);
        (addr, (effective & 0xFF00) != (addr & 0xFF00))
    }

    fn ind_abs(&mut self) -> u16 {
        let addr = self.abs();
        let lo = self.bus.read(addr) as u16;
        let hi = if (addr & 0x00FF) == 0x00FF {
            self.bus.read(addr & 0xFF00) as u16
        } else {
            self.bus.read(addr + 1) as u16
        };
        (hi << 8) | lo
    }

    fn rel(&mut self) -> i8 {
        self.imm() as i8
    }

    fn imp(&mut self) {
        // No operation needed
    }

    // Interrupt handling
    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn trigger_irq(&mut self) {
        if !self.get_flag(INTERRUPT_DISABLE) {
            self.irq_pending = true;
        }
    }

    fn handle_interrupt(&mut self, int_type: InterruptType) -> usize {
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);

        let mut status = self.status | 0x20;
        if int_type == InterruptType::Brk {
            status |= BREAK;
        }
        self.push(status);

        self.set_flag(INTERRUPT_DISABLE, true);

        let vector = match int_type {
            InterruptType::Nmi => 0xFFFA,
            InterruptType::Irq | InterruptType::Brk => 0xFFFE,
        };

        self.pc = self.read_u16(vector);
        7
    }

    // Instruction implementations
    fn lda(&mut self, value: u8) {
        self.a = value;
        self.set_flag(ZERO, self.a == 0);
        self.set_flag(NEGATIVE, (self.a & 0x80) != 0);
    }

    fn sta(&mut self, addr: u16) {
        self.bus.write(addr, self.a);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_flag(ZERO, self.x == 0);
        self.set_flag(NEGATIVE, (self.x & 0x80) != 0);
    }

    // Main execution loop
    pub fn step(&mut self) -> usize {
        let mut cycles = 0;

        // Handle interrupts
        if self.nmi_pending {
            self.nmi_pending = false;
            return self.handle_interrupt(InterruptType::Nmi);
        }

        if self.irq_pending && !self.get_flag(INTERRUPT_DISABLE) {
            self.irq_pending = false;
            return self.handle_interrupt(InterruptType::Irq);
        }

        // Fetch and execute instruction
        let opcode = self.bus.read(self.pc);
        self.pc += 1;

        match opcode {
            // LDA Immediate
            0xA9 => {
                let value = self.imm();
                self.lda(value);
                cycles = 2;
            }
            
            // STA Absolute
            0x8D => {
                let addr = self.abs();
                self.sta(addr);
                cycles = 4;
            }
            
            // TAX
            0xAA => {
                self.tax();
                cycles = 2;
            }
            
            // BRK
            0x00 => {
                self.pc += 1;
                cycles = self.handle_interrupt(InterruptType::Brk) + 1;
            }
            
            // Unimplemented opcode handler
            _ => panic!("Unimplemented opcode: {:#04X}", opcode),
        }

        cycles
    }
}
