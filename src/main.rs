extern crate sdl2;

mod audio_driver;
mod disassembler;
mod font;
mod input_driver;
mod memory;
mod processor;
mod video_driver;


use audio_driver::AudioDriver;
use disassembler::{NullDisassembler};
use input_driver::InputDriver;
use memory::MemoryBus;
use processor::Processor;
use std::{env, time::Duration, thread};
use video_driver::VideoDriver;

const CHIP_8_WIDTH: usize = 64;
const CHIP_8_HEIGHT: usize = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &str = if args.len() != 2 {
        "/home/hermes/cpp/Chip-8/ROMS/programs/Chip8 Picture.ch8"
    } else {
        &args[1]
    };

    let sdl_context = sdl2::init().unwrap();

    let mut video_driver = VideoDriver::new(&sdl_context);
    let audio_driver = AudioDriver::new(&sdl_context);
    let mut input = InputDriver::new(&sdl_context);

    let mut bus = MemoryBus::new();

    bus.load_rom(filename);

    let disassembler = Box::new(NullDisassembler{});
    let mut cpu = Processor::new(bus, disassembler);

    while input.poll() {
        let output = cpu.tick(&input.keyboard);

        if output.vram_changed {
            video_driver.draw(output.vram);
        }

        // make sound 
        audio_driver.beep(output.beep);

        thread::sleep(Duration::from_millis(100));
    }
}
