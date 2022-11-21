#![allow(dead_code, unused)]
use rand::Rng;
use std::{fmt::Display, ops::{Shr, BitAnd, Shl, BitOr, BitAndAssign, BitOrAssign, BitXorAssign}};

use crate::{memory::MemoryBus, disassembler::Disassembler, CHIP_8_WIDTH, CHIP_8_HEIGHT};

const CHIP_8_STACK_SIZE: usize = 16;
const CHIP_8_REGISTERS: usize = 16;

pub struct ProcessorOutput<'a> {
    pub vram: &'a [[u8; CHIP_8_WIDTH]; CHIP_8_HEIGHT],
    pub vram_changed: bool,
    pub beep: bool
}

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
    sound_timer: u8,
    delay_timer: u8,
    vram_changed: bool,
    // Area for the stack
    stack: [u16; 16],
    vram: [[u8; 64]; 32],
    // Keyboard specific
    wait_for_key: bool,
    saved_key_state: [bool; 16],
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
            stack: [0; CHIP_8_STACK_SIZE],
            v: [0; CHIP_8_REGISTERS],
            reg_i: 0, 
            sound_timer: 0,
            delay_timer: 0,
            vram_changed: false,
            vram: [[0 ; CHIP_8_WIDTH]; CHIP_8_HEIGHT],
            wait_for_key: false,
            saved_key_state: [false; 16],
            bus: membus,
            disassembler: dis,
        } 
    }

    pub fn tick(&mut self, keyboard: &[bool; 16]) -> ProcessorOutput {
        self.vram_changed = false;
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
        self.execute_opcode(opcode, keyboard)
    }
    
    fn execute_opcode(&mut self, opcode: u16, keyboard: &[bool; 16]) -> ProcessorOutput {
        self.disassembler.disassemble(opcode);
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );
        let addr = opcode & 0x0FFF;
        let vx = nibbles.1 as usize;
        let vy = nibbles.2 as usize;
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
            (0x8,   _,   _, 0x4) => self.op_add_vx_vy(vx, vy),
            (0x8,   _,   _, 0x5) => self.op_sub_vx_vy(vx, vy),
            (0x8,   _,   _, 0x6) => self.op_shr_vx(vx),
            (0x8,   _,   _, 0x7) => self.op_sub_vx_vy(vy, vx),
            (0x8,   _,   _, 0xE) => self.op_shl_vx(vx),
            (0x9,   _,   _, 0x0) => self.op_skip_neq_vx_vy(vx, vy),
            (0xA,   _,   _,   _) => self.op_mov_i(addr),
            (0xB,   _,   _,   _) => todo!(),
            (0xC,   _,   _,   _) => self.op_rand(vx, kk),
            (0xD,   _,   _,   _) => self.op_draw(vx, vy, nibbles.3),
            (0xE,   _, 0x9, 0xE) => self.op_skip_key_eq_vx(vx, keyboard),
            (0xE,   _, 0xA, 0x1) => self.op_skip_key_neq_vx(vx, keyboard),
            (0xF,   _, 0x0, 0x7) => self.op_ld_vx_delay(vx),
            (0xF,   _, 0x0, 0xA) => todo!(),
            (0xF,   _, 0x1, 0x5) => self.op_set_delay(vx),
            (0xF,   _, 0x1, 0x8) => self.op_set_sound(vx),
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
        ProcessorOutput { vram: &self.vram, vram_changed: self.vram_changed, beep: {self.sound_timer > 0} }
    }
}

// Opcodes
impl Processor {
    fn op_nop(&self) -> ProcessorAction {
        ProcessorAction::Next
    }

    fn op_cls(&mut self) -> ProcessorAction {
        for row in 0..CHIP_8_HEIGHT {
            for col in 0..CHIP_8_WIDTH {
                self.vram[row][col] = 0;
            }
        }
        self.vram_changed = true;
        ProcessorAction::Next
    }

    fn op_ret(&mut self) -> ProcessorAction {
        // pop address from stack
        self.pc = self.stack[self.sp];
        // dec stack pointer
        self.sp -= 1;
        ProcessorAction::Next
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

    fn op_skip_eq_vx_kk(&self, vx: usize, kk: u8) -> ProcessorAction {
        if self.v[vx] == kk {
            return ProcessorAction::Skip
        }
        ProcessorAction::Next
    }

    fn op_skip_neq_vx_kk(&self, vx: usize, kk: u8) -> ProcessorAction {
        if self.v[vx] != kk {
            return ProcessorAction::Skip
        }
        ProcessorAction::Next
    }

    fn op_skip_eq_vx_vy(&self, vx: usize, vy: usize) -> ProcessorAction {
        if self.v[vx] == self.v[vy] {
            return ProcessorAction::Skip
        }
        ProcessorAction::Next
    }

    fn op_mov_vx_kk(&mut self, vx: usize, kk:u8) -> ProcessorAction {
        self.v[vx] = kk;
        ProcessorAction::Next
    }

    fn op_or_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        self.v[vx].bitor_assign(self.v[vy]);
        ProcessorAction::Next
    }

    fn op_and_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        self.v[vx].bitand_assign(self.v[vy]);
        ProcessorAction::Next
    }

    fn op_xor_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        self.v[vx].bitxor_assign(self.v[vy]);
        ProcessorAction::Next
    }

    fn op_add_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        let (res, carry) = self.v[vx].overflowing_add(self.v[vy]);
        self.v[vx] = res;
        self.v[0xf_usize] = if carry {1} else {0};
        ProcessorAction::Next
    }

    fn op_sub_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        let (res, borrow) = self.v[vx].overflowing_sub(self.v[vy]);
        self.v[vx] = res;
        self.v[0xf_usize] = if borrow {0} else {1};
        ProcessorAction::Next
    }

    fn op_add_vx_kk(&mut self, vx: usize, kk: u8) -> ProcessorAction {
        self.v[vx] = self.v[vx].wrapping_add(kk);
        ProcessorAction::Next
    }

    fn op_mov_vx_vy(&mut self, vx: usize, vy: usize) -> ProcessorAction {
        self.v[vx] = self.v[vy];
        ProcessorAction::Next
    }

    fn op_shr_vx(&mut self, vx: usize) -> ProcessorAction {
        let x = self.v[vx];
        self.v[0xf_usize] = x.bitand(0x1);
        self.v[vx] = x.shr(1);
        ProcessorAction::Next
    }

    fn op_shl_vx(&mut self, vx: usize) -> ProcessorAction {
        let x = self.v[vx];
        self.v[0xf_usize] = x.bitand(0x80) >> 7; // Check msb
        self.v[vx] = x.shl(1);
        ProcessorAction::Next
    }

    fn op_skip_neq_vx_vy(&self, vx: usize, vy: usize) -> ProcessorAction {
        if self.v[vx] != self.v[vy] {
            return ProcessorAction::Skip;
        }
        ProcessorAction::Next
    }

    fn op_mov_i(&mut self, addr: u16) -> ProcessorAction {
        self.reg_i = addr;
        ProcessorAction::Next
    }

    fn op_rand(&mut self, vx: usize, kk: u8) -> ProcessorAction {
        let mut rng = rand::thread_rng();
        let random: u8 = rng.gen();
        self.v[vx] = random.bitand(kk);
        ProcessorAction::Next
    }

    fn op_draw(&mut self, vx: usize, vy: usize, n: u8) -> ProcessorAction {
        self.v[0xf_usize] = 0;

        for line in 0..n {
            let y = ((self.v[vy] + line) as usize) % CHIP_8_HEIGHT;
            let sprite = self.bus.read_byte(self.reg_i + (line as u16));

            for bit in 0..8_u8{
                let x = ((self.v[vx] + bit) as usize) % CHIP_8_WIDTH;
                // check for collision
                if sprite.shr(7-bit).bitand(1) == 1 {
                    if self.vram[y][x] == 0xFF {
                        self.v[0xf_usize] = 1;
                    }
                    self.vram[y][x] ^= 0xFF;
                }
            }
        }
        
        self.vram_changed = true;
        ProcessorAction::Next
    }

    fn op_skip_key_eq_vx(&mut self, vx: usize, keyboard: &[bool; 16]) -> ProcessorAction {
        // if key down skip the next instruction
        if keyboard[self.v[vx] as usize] {
            return ProcessorAction::Skip;
        }
        ProcessorAction::Next
    }

    fn op_skip_key_neq_vx(&mut self, vx: usize, keyboard: &[bool; 16]) -> ProcessorAction {
        // if key up skip the next instruction
        if !keyboard[self.v[vx] as usize] {
            return  ProcessorAction::Skip;
        }
        ProcessorAction::Next
    }

    fn op_ld_vx_delay(&mut self, vx: usize) -> ProcessorAction {
        self.v[vx] = self.delay_timer;
        ProcessorAction::Next
    }

    fn op_set_delay(&mut self, vx: usize) -> ProcessorAction {
        self.delay_timer = self.v[vx];
        ProcessorAction::Next
    }

    fn op_set_sound(&mut self, vx: usize) -> ProcessorAction {
        self.sound_timer = self.v[vx];
        ProcessorAction::Next
    }

    fn op_add_i_vx(&mut self, vx: usize) -> ProcessorAction {
        self.reg_i = self.reg_i.wrapping_add(self.v[vx].into());
        ProcessorAction::Next
    }

    fn op_bcd(&mut self, vx: usize) -> ProcessorAction {
        let val = self.v[vx];
        self.bus.write(self.reg_i+2, (val % 100) % 10);
        self.bus.write(self.reg_i+1, (val / 10) % 10);
        self.bus.write(self.reg_i, (val / 100));

        ProcessorAction::Next
    }

    fn op_ld_i_mem_vx(&mut self, vx: usize) -> ProcessorAction {
        self.reg_i = (self.v[vx] as u16 * 5u16) & 0x0FFF;
        ProcessorAction::Next
    }

    fn op_ld_mem_i_vx(&mut self, vx: usize) -> ProcessorAction {
        let val = self.v[vx];
        self.bus.write(self.reg_i, val);
        ProcessorAction::Next
    }

    fn op_ld_vx_mem_i(&mut self, vx: usize) -> ProcessorAction {
        self.v[vx] = self.bus.read_byte(self.reg_i);
        ProcessorAction::Next
    }
}

#[cfg(test)]

use crate::disassembler::NullDisassembler;

#[test]
fn test_op_shr() {
    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    let keyboard = [false; 16];

    cpu.v[0] = 5;
    cpu.execute_opcode(0x8006, &keyboard);
    assert_eq!(cpu.v[0], 2);
    assert_eq!(cpu.v[0xf], 1);
}

#[test]
fn test_op_shl() {
    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    let keyboard = [false; 16];

    cpu.v[0] = 255;
    cpu.execute_opcode(0x800E, &keyboard);
    assert_eq!(cpu.v[0], 0b1111_1110);
    assert_eq!(cpu.v[0xf], 1);
}

#[test]
fn test_op_or() {
    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    let keyboard = [false; 16];

    cpu.v[0] = 255;
    cpu.v[1] = 0;
    cpu.execute_opcode(0x8011, &keyboard);
    assert_eq!(cpu.v[0], 0b1111_1111);
}

#[test]
fn test_op_and() {
    let bus = MemoryBus::new();
    let mut cpu = Processor::new(bus, Box::new(NullDisassembler{}));
    let keyboard = [false; 16];

    cpu.v[0] = 255;
    cpu.v[1] = 0;
    cpu.execute_opcode(0x8012, &keyboard);
    assert_eq!(cpu.v[0], 0);
}