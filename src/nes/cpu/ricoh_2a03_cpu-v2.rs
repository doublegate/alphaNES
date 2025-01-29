// src/nes/cpu/ricoh_2a03_cpu.rs
// Ricoh 2A03 CPU Core (NES) Implementation

use crate::nes::cpu::Bus;

// Status Flags
const CARRY: u8 = 1 << 0;
const ZERO: u8 = 1 << 1;
const INTERRUPT_DISABLE: u8 = 1 << 2;
const DECIMAL: u8 = 1 << 3;
const BREAK: u8 = 1 << 4;
const OVERFLOW: u8 = 1 << 6;
const NEGATIVE: u8 = 1 << 7;

// Interrupt Types
#[derive(PartialEq)]
enum InterruptType {
    Nmi,
    Irq,
    Brk,
}

/// Ricoh 2A03 CPU Core
pub struct Cpu2A03<B: Bus> {
    // Registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: u8,

    // Interrupt State
    pub nmi_pending: bool,
    pub irq_pending: bool,
    pub interrupt_mask_delay: bool,

    // Memory Bus
    pub bus: B,

    // Cycle Counting
    cycles: usize,
}

impl<B: Bus> Cpu2A03<B> {
    // Initialization
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

    // Memory Operations
    fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.bus.read(addr) as u16;
        let hi = self.bus.read(addr + 1) as u16;
        (hi << 8) | lo
    }

    // Stack Operations
    fn push(&mut self, data: u8) {
        self.bus.write(0x0100 | self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read(0x0100 | self.sp as u16)
    }

    // Flag Management
    fn set_flag(&mut self, flag: u8, condition: bool) {
        self.status = if condition {
            self.status | flag
        } else {
            self.status & !flag
        };
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) != 0
    }

    // Addressing Modes
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

    // Interrupt Handling
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

        let mut status = self.status | 0x20; // Unused flag always set
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
        7 // Interrupt cycle count
    }

    // Core Instructions
    fn adc(&mut self, value: u8) {
        // NES-specific: Decimal mode disabled
        let sum = self.a as u16 + value as u16 + self.get_flag(CARRY) as u16;
        self.set_flag(CARRY, sum > 0xFF);
        self.set_flag(OVERFLOW, ((self.a ^ sum as u8) & (value ^ sum as u8) & 0x80) != 0);
        self.a = sum as u8;
        self.set_flag(ZERO, self.a == 0);
        self.set_flag(NEGATIVE, (self.a & 0x80) != 0);
    }

    fn sbc(&mut self, value: u8) {
        // NES-specific: Decimal mode disabled
        let value = !value as u16;
        let sum = self.a as u16 + value + self.get_flag(CARRY) as u16;
        self.set_flag(CARRY, sum > 0xFF);
        self.set_flag(OVERFLOW, ((self.a ^ sum as u8) & (!value as u8 ^ sum as u8) & 0x80) != 0);
        self.a = sum as u8;
        self.set_flag(ZERO, self.a == 0);
        self.set_flag(NEGATIVE, (self.a & 0x80) != 0);
    }

    // Unofficial Opcode Helpers
    fn alr(&mut self, value: u8) {
        self.a &= value;
        self.set_flag(CARRY, (self.a & 0x01) != 0);
        self.a >>= 1;
        self.set_flag(ZERO, self.a == 0);
        self.set_flag(NEGATIVE, false);
    }

    fn dcp(&mut self, addr: u16) {
        let mut value = self.bus.read(addr);
        value = value.wrapping_sub(1);
        self.bus.write(addr, value);
        self.compare(self.a, value);
    }

    // Main Execution Loop
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

        // Fetch and execute
        let opcode = self.bus.read(self.pc);
        self.pc += 1;

        match opcode {
            // Official Opcodes
            0xA9 => { // LDA Imm
                self.a = self.imm();
                self.set_flag(ZERO, self.a == 0);
                self.set_flag(NEGATIVE, (self.a & 0x80) != 0);
                cycles = 2;
            }

            0x8D => { // STA Abs
                let addr = self.abs();
                self.bus.write(addr, self.a);
                cycles = 4;
            }

            // ... Add remaining official opcodes ...

            // Unofficial Opcodes
            0x4B => { // ALR Imm
                let value = self.imm();
                self.alr(value);
                cycles = 2;
            }

            0xC7 => { // DCP Zpg
                let addr = self.zpg();
                self.dcp(addr);
                cycles = 5;
            }

            // ... Add remaining unofficial opcodes ...

            _ => match opcode {
                // NOP variants
                0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => cycles = 2,
                
                // Undocumented NOPs with operand reads
                0x04 | 0x44 | 0x64 => {
                    let _ = self.zpg();
                    cycles = 3;
                }
                
                _ => panic!("Unimplemented opcode: {:#04X}", opcode),
            }
        }

        self.cycles += cycles;
        cycles
    }

    // Helper Functions
    fn compare(&mut self, reg: u8, value: u8) {
        let result = reg.wrapping_sub(value);
        self.set_flag(CARRY, reg >= value);
        self.set_flag(ZERO, result == 0);
        self.set_flag(NEGATIVE, (result & 0x80) != 0);
    }
}
