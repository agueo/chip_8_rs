pub trait Disassembler {
    fn disassemble(&self, opcode: u16);
}

#[derive(Debug)]
pub struct NullDisassembler {}
impl Disassembler for NullDisassembler {
    fn disassemble(&self, _opcode: u16) {}
}

#[derive(Debug)]
pub struct DebugDisassembler{}
impl Disassembler for DebugDisassembler {
    fn disassemble(&self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );
        let addr = opcode & 0xFFF;
        let vx = nibbles.1;
        let vy = nibbles.2;
        let kk = (opcode & 0xFF) as u16;

        match nibbles {
            (0x0, 0x0, 0x0, 0x0) => println!("{:#06x}: Nop", opcode),
            (0x0, 0x0, 0xE, 0x0) => println!("{:#06x}: CLS", opcode),
            (0x0, 0x0, 0xE, 0xE) => println!("{:#06x}: RET", opcode),
            (0x1,   _,   _,   _) => println!("{:#06x}: JMP {}", opcode, addr),
            (0x2,   _,   _,   _) => println!("{:#06x}: CALL {}", opcode, addr),
            (0x3,   _,   _,   _) => println!("{:#06x}: SKIP v{} == {}", opcode, vx, kk), 
            (0x4,   _,   _,   _) => println!("{:#06x}: SKIP v{} != {}", opcode, vx, kk), 
            (0x5,   _,   _, 0x0) => println!("{:#06x}: SKIP v{} == v{}", opcode, vx, vy), 
            (0x6,   _,   _,   _) => println!("{:#06x}: MOV v{}, {}", opcode, vx, kk),
            (0x7,   _,   _,   _) => println!("{:#06x}: ADD v{}, {}", opcode, vx, kk),
            (0x8,   _,   _, 0x0) => println!("{:#06x}: MOV v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x1) => println!("{:#06x}: OR v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x2) => println!("{:#06x}: AND v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x3) => println!("{:#06x}: XOR v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x4) => println!("{:#06x}: ADC v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x5) => println!("{:#06x}: SBC v{}, v{}", opcode, vx, vy),
            (0x8,   _,   _, 0x6) => println!("{:#06x}: SHR v{}", opcode, vx),
            (0x8,   _,   _, 0x7) => println!("{:#06x}: SBC v{}, v{}", opcode, vy, vx),
            (0x8,   _,   _, 0xE) => println!("{:#06x}: SHL v{}", opcode, vx),
            (0x9,   _,   _, 0x0) => println!("{:#06x}: SKIP v{} != v{}", opcode, vx, vy), 
            (0xA,   _,   _,   _) => println!("{:#06x}: MOV I, {}", opcode, addr),
            (0xB,   _,   _,   _) => println!("{:#06x}: JMP V0, {}", opcode, addr),
            (0xC,   _,   _,   _) => println!("{:#06x}: RAND v{}, {}", opcode, vx, kk),
            (0xD,   _,   _,   _) => println!("{:#06x}: DRAW v{}, v{}, {}", opcode, vx, vy, nibbles.3),
            (0xE,   _, 0x9, 0xE) => println!("{:#06x}: SKIP KEY == v{}", opcode, vx),
            (0xE,   _, 0xA, 0x1) => println!("{:#06x}: SKIP KEY != v{}", opcode, vx),
            (0xF,   _, 0x0, 0x7) => println!("{:#06x}: LD v{}, DT", opcode, vx),
            (0xF,   _, 0x0, 0xA) => println!("{:#06x}: LD v{}, key", opcode, vx), // wait for key press
            (0xF,   _, 0x1, 0x5) => println!("{:#06x}: LD DT, v{}", opcode, vx),
            (0xF,   _, 0x1, 0x8) => println!("{:#06x}: LD ST, v{}", opcode, vx),
            (0xF,   _, 0x1, 0xE) => println!("{:#06x}: ADD I, v{}", opcode, vx),
            (0xF,   _, 0x2, 0x9) => println!("{:#06x}: LD I, [v{}]", opcode, vx),
            (0xF,   _, 0x3, 0x3) => println!("{:#06x}: BCD I, v{}", opcode, vx),
            (0xF,   _, 0x5, 0x5) => println!("{:#06x}: LD [I], v{}", opcode, vx),
            (0xF,   _, 0x6, 0x5) => println!("{:#06x}: LD v{}, [I]", opcode, vx),
            _ => println!("Invalid opcode: {}", opcode),
        };       
    }
}