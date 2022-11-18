#![allow(unused, unused_imports)]

mod disassembler;
mod processor;
mod memory;
mod font;

use disassembler::{DebugDisassembler, Disassembler};
use memory::MemoryBus;
use processor::Processor;
// use font::FONT;


fn main() {
    let test_program = "/home/hermes/cpp/Chip-8/ROMS/games/Brick (Brix hack, 1990).ch8".to_owned();
    let disassembler = Box::new(DebugDisassembler{});
    let mut bus = MemoryBus::new();
    bus.load_rom(test_program);

    let mut cpu = Processor::new(bus, disassembler);

    let mut running = true;
    while running {
        cpu.tick();
        println!("cpu: {}", cpu);
    }
}