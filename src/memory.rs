#![allow(unused)]

use std::fs;
use crate::font::FONT;

#[derive(Debug)]
pub struct MemoryBus{
    rom: [u8; 4096],
    vram: Vec<u8>,
}

const ROM_SIZE: usize = 4016;

impl MemoryBus{
    pub fn new() -> Self {
        let mut rom= [0;4096];
        for (i, data) in FONT.into_iter().enumerate() {
            rom[i] = data;
        }

        MemoryBus { rom, vram: [0;64*32].to_vec() }
    }

    pub fn load_rom(&mut self, filename: String) {
        let rom_data = load_rom(filename);
        let start_offset = 512;
        for (i, data) in rom_data.into_iter().enumerate() {
            if i > 3584 { break; }
            self.rom[start_offset + i] = data;
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        *self.rom.get(addr as usize).unwrap()
    }

    pub fn read_word(&self, addr: u16) -> u16{
        let hi = *self.rom.get(addr as usize).unwrap() as u16;
        let lo = *self.rom.get(addr as usize + 1).unwrap() as u16;
        return ((hi << 8) | lo) as u16;
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.rom[addr as usize] = val;
    }

}

pub fn load_rom(filename: String) -> Vec<u8> {
    let data = fs::read(filename).unwrap();
    data
}



#[cfg(test)]
#[test]
fn test_create() {
    let bus = MemoryBus::new();
    println!("rom length:{}", bus.rom.len());
    assert_eq!(bus.rom.len(), 4096);
    assert_eq!(bus.vram.len(), 64*32);
}

#[test]
fn test_read() {
    let mut bus: MemoryBus = MemoryBus::new();

    let read_value = bus.read_byte(0x00);
    assert_eq!(read_value, FONT[0]);
}

#[test]
fn test_write() {
    let mut bus  = MemoryBus::new();
    bus.write(0x200, 32);
    let val_read = bus.read_byte(0x200);
    assert_eq!(val_read, 32);
}

#[test]
fn test_load_rom() {
    let mut bus = MemoryBus::new();
    let test_program = "/home/hermes/cpp/Chip-8/ROMS/programs/BMP Viewer - Hello (C8 example) [Hap, 2005].ch8".to_owned();
    bus.load_rom(test_program);

    assert_eq!(bus.rom[512], 0x12);
}