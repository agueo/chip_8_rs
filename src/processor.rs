#![allow(dead_code, unused)]
use std::{fmt::Display, ops::{Shr, BitAnd, Shl, BitOr, BitAndAssign, BitOrAssign, BitXorAssign}};

use crate::{memory::MemoryBus, disassembler::Disassembler};

enum ProcessorAction {
    Next,
    Skip,
    Jump(u16)
}

pub struct Processor{
    // Registers
    pc: u16,
    sp: usize,
    v: [u8;16],
    reg_i: u16,
    sound_timer: u16,
    delay_timer: u16,
    // Area for the stack
    stack: [u16; 16],

    // Memory bus
    bus: MemoryBus,
    // Disassembler object
    disassembler: Box<dyn Disassembler>,
}

impl Display for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v: {:?}\npc: {}\nsp: {}\nI: {}\nST: {}\nDT: {}",
            self.v, self.pc, self.sp, self.reg_i, self.sound_timer, self.delay_timer)
    }
}

impl Processor {
    pub fn new(membus: MemoryBus, dis: Box<dyn Disassembler>) -> Self {
        Processor { 
            pc: 0x200, 
            sp: 0, 
            stack: [0;16],
            v: [0;16],
            reg_i: 0, 
            sound_timer: 0,
            delay_timer: 0,
            bus: membus,
            disassembler: dis,
        } 
    }

    pub fn tick(&mut self) {
        // TODO Check keyboard

        // TODO decrement timers
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        // chip-8 is big endian
        let opcode = self.bus.read_word(self.pc);
        self.execute_opcode(opcode);
    }
    
    fn execute_opcode(&mut self, opcode: u16) {
        self.disassembler.disassemble(opcode);
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            ((opcode & 0x000F) >> 0) as u8,
        );
        let addr = opcode & 0xFFF;
        let vx = nibbles.1;
        let vy = nibbles.2;
        let kk = (opcode & 0xFF) as u8;
        
        let action = match nibbles {
            (0x0, 0x0, 0x0, 0x0) => self.op_nop(),
            (0x0, 0x0, 0xE, 0x0) => self.op_cls(),
            (0x0, 0x0, 0xE, 0xE) => self.op_ret(),
            (0x1,   _,   _,   _) => self.op_jmp(addr),
            (0x2,   _,   _,   _) => self.op_call(addr),
            (0x3,   _,   _,   _) => self.op_skip_eq_vx_kk(vx, kk),
            (0x4,   _,   _,   _) => self.op_skip_neq_vx_kk(vx, kk),
            (0x5,   _,   _, 0x0) => self.op_skip_eq_vx_vy(vx, vy),
            (0x6,   _,   _,   _) => self.op_mov_vx_kk(vx, kk),
            (0x7,   _,   _,   _) => self.op_add_vx_kk(vx, kk),
            (0x8,   _,   _, 0x0) => self.op_mov_vx_vy(vx, vy),
            (0x8,   _,   _, 0x1) => self.op_or_vx_vy(vx, vy),
            (0x8,   _,   _, 0x2) => self.op_and_vx_vy(vx, vy),
            (0x8,   _,   _, 0x3) => self.op_xor_vx_vy(vx, vy),
            (0x8,   _,   _, 0x4) => todo!(),
            (0x8,   _,   _, 0x5) => todo!(),
            (0x8,   _,   _, 0x6) => self.op_shr_vx(vx),
            (0x8,   _,   _, 0x7) => todo!(),
            (0x8,   _,   _, 0xE) => self.op_shl_vx(vx),
            (0x9,   _,   _, 0x0) => self.op_skip_neq_vx_vy(vx, vy),
            (0xA,   _,   _,   _) => self.op_mov_i(addr),
            (0xB,   _,   _,   _) => todo!(),
            (0xC,   _,   _,   _) => todo!(),
            (0xD,   _,   _,   _) => self.op_draw(vx, vy, kk),
            (0xE,   _, 0x9, 0xE) => todo!(),
            (0xE,   _, 0xA, 0x1) => todo!(),
            (0xF,   _, 0x0, 0x7) => todo!(),
            (0xF,   _, 0x0, 0xA) => todo!(),
            (0xF,   _, 0x1, 0x5) => todo!(),
            (0xF,   _, 0x1, 0x8) => todo!(),
            (0xF,   _, 0x1, 0xE) => self.op_add_i_vx(vx),
            (0xF,   _, 0x2, 0x9) => self.op_ld_i_mem_vx(vx),
            (0xF,   _, 0x3, 0x3) => self.op_bcd(vx),
            (0xF,   _, 0x5, 0x5) => self.op_ld_mem_i_vx(vx),
            (0xF,   _, 0x6, 0x5) => self.op_ld_vx_mem_i(vx),
            _ => {println!("Invalid opcode: {}", opcode); ProcessorAction::Next},
        };

        match action {
            ProcessorAction::Next => self.pc += 2,
            ProcessorAction::Skip => self.pc += 4,
            ProcessorAction::Jump(addr) => self.pc = addr,
        }
    }
}

// Opcodes
impl Processor {
    fn op_nop(&self) -> ProcessorAction {
        ProcessorAction::Next
    }

    fn op_cls(&self) -> ProcessorAction {
        // TODO
        println!("Clearing Screen");
        ProcessorAction::Next
    }

    fn op_ret(&mut self) -> ProcessorAction {
        // pop address from stack
        let addr = self.stack[self.sp];
        // dec stack pointer
        self.sp -= 1;
        ProcessorAction::Jump(addr)
    }

    fn op_jmp(&self, addr: u16) -> ProcessorAction {
        ProcessorAction::Jump(addr)
    }

    fn op_call(&mut self, addr: u16) -> ProcessorAction {
        // inc stack_pointer
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        ProcessorAction::Jump(addr)
    }

    fn op_skip_eq_vx_kk(&self, vx: u8, kk: u8) -> ProcessorAction {
        if self.v[vx as usize] == kk {
            return ProcessorAction::Skip
        }
        ProcessorAction::Next
    }

    fn op_skip_neq_vx_kk(&self, vx: u8, kk: u8) -> ProcessorAction {
        if self.v[vx as usize] != kk {
            return ProcessorAction::Skip
        }
        ProcessorAction::Next
    }

    fn op_skip_eq_vx_vy(&self, vx: u8, vy: u8) -> ProcessorAction {
        if self.v[vx as usize] == self.v[vy as usize] {
            return ProcessorAction::Skip
        }
        return ProcessorAction::Next
    }

    fn op_mov_vx_kk(&mut self, vx: u8, kk:u8) -> ProcessorAction {
        self.v[vx as usize] = kk;
        ProcessorAction::Next
    }

    fn op_or_vx_vy(&mut self, vx: u8, vy: u8) -> ProcessorAction {
        self.v[vx as usize].bitor_assign(self.v[vy as usize]);
        ProcessorAction::Next
    }

    fn op_and_vx_vy(&mut self, vx: u8, vy: u8) -> ProcessorAction {
        self.v[vx as usize].bitand_assign(self.v[vy as usize]);
        ProcessorAction::Next
    }

    fn op_xor_vx_vy(&mut self, vx: u8, vy: u8) -> ProcessorAction {
        self.v[vx as usize].bitxor_assign(self.v[vy as usize]);
        ProcessorAction::Next
    }

    fn op_add_vx_kk(&mut self, vx: u8, kk: u8) -> ProcessorAction {
        self.v[vx as usize] = self.v[vx as usize].wrapping_add(kk);
        ProcessorAction::Next
    }

    fn op_mov_vx_vy(&mut self, vx: u8, vy:u8) -> ProcessorAction {
        self.v[vx as usize] = self.v[vy as usize];
        ProcessorAction::Next
    }

    fn op_shr_vx(&mut self, vx: u8) -> ProcessorAction {
        let x = self.v[vx as usize];
        self.v[0xf as usize] = x.bitand(0x1);
        self.v[vx as usize] = x.shr(1);
        ProcessorAction::Next
    }

    fn op_shl_vx(&mut self, vx: u8) -> ProcessorAction {
        let x = self.v[vx as usize];
        self.v[0xf as usize] = x.bitand(0x80) >> 7; // Check msb
        self.v[vx as usize] = x.shl(1);
        ProcessorAction::Next
    }

    fn op_skip_neq_vx_vy(&self, vx: u8, vy: u8) -> ProcessorAction {
        if self.v[vx as usize] != self.v[vy as usize] {
            return ProcessorAction::Skip;
        }
        ProcessorAction::Next
    }

    fn op_mov_i(&mut self, addr: u16) -> ProcessorAction {
        self.reg_i = addr;
        ProcessorAction::Next
    }

    fn op_draw(&mut self, vx: u8, vy: u8, n: u8) -> ProcessorAction {
        // TODO
        ProcessorAction::Next
    }

    fn op_add_i_vx(&mut self, vx: u8) -> ProcessorAction {
        self.reg_i = self.reg_i.wrapping_add(self.v[vx as usize].into());
        ProcessorAction::Next
    }

    fn op_bcd(&mut self, vx: u8) -> ProcessorAction {
        let val = self.v[vx as usize];
        self.bus.write(self.reg_i+2, (val % 100) % 10);
        self.bus.write(self.reg_i+1, (val / 10) % 10);
        self.bus.write(self.reg_i, (val / 100));

        ProcessorAction::Next
    }

    fn op_ld_i_mem_vx(&mut self, vx: u8) -> ProcessorAction {
        self.reg_i = self.bus.read_word(self.v[vx as usize].into());
        ProcessorAction::Next
    }

    fn op_ld_mem_i_vx(&mut self, vx: u8) -> ProcessorAction {
        let val = self.v[vx as usize];
        self.bus.write(self.reg_i, val);
        ProcessorAction::Next
    }

    fn op_ld_vx_mem_i(&mut self, vx: u8) -> ProcessorAction {
        self.v[vx as usize] = self.bus.read_byte(self.reg_i);
        ProcessorAction::Next
    }
}

#[cfg(test)]
#[test]
fn test_op_shr() {
    use crate::disassembler::NullDisassembler;

    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    cpu.v[0] = 5;
    cpu.execute_opcode(0x8006);
    assert_eq!(cpu.v[0], 2);
    assert_eq!(cpu.v[0xf], 1);

}

#[test]
fn test_op_shl() {
    use crate::disassembler::NullDisassembler;

    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    cpu.v[0] = 255;
    cpu.execute_opcode(0x800E);
    assert_eq!(cpu.v[0], 0b1111_1110);
    assert_eq!(cpu.v[0xf], 1);
}

#[test]
fn test_op_or() {
    use crate::disassembler::NullDisassembler;

    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    cpu.v[0] = 255;
    cpu.v[1] = 0;
    cpu.execute_opcode(0x8011);
    assert_eq!(cpu.v[0], 0b1111_1111);
}

#[test]
fn test_op_and() {
    use crate::disassembler::NullDisassembler;

    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    cpu.v[0] = 255;
    cpu.v[1] = 0;
    cpu.execute_opcode(0x8012);
    assert_eq!(cpu.v[0], 0);
}