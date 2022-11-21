use sdl2::{event::Event, Sdl, EventPump};

pub struct InputDriver {
    events: EventPump,
    pub keyboard: [bool; 16]
}

impl InputDriver {
    pub fn new(sdl_context: &Sdl) -> Self{
        let events = sdl_context.event_pump().unwrap();
    
        Self{events, keyboard: [false; 16]}
    }

    pub fn poll(&mut self) -> bool {
        use sdl2::keyboard::Keycode;

        for event in self.events.poll_iter() {
            match event {
                Event::Quit {..} => { return false },
                Event::KeyDown {keycode, ..} => {
                    match keycode.unwrap() {
                        Keycode::Num1 => { self.keyboard[0x1] = true; },
                        Keycode::Num2 => { self.keyboard[0x2] = true; },
                        Keycode::Num3 => { self.keyboard[0x3] = true; },
                        Keycode::Num4 => { self.keyboard[0xC] = true; },
                        Keycode::Q => { self.keyboard[0x4] = true; },
                        Keycode::W => { self.keyboard[0x5] = true; },
                        Keycode::E => { self.keyboard[0x6] = true; },
                        Keycode::R => { self.keyboard[0xD] = true; },
                        Keycode::A => { self.keyboard[0x7] = true; },
                        Keycode::S => { self.keyboard[0x8] = true; },
                        Keycode::D => { self.keyboard[0x9] = true; },
                        Keycode::F => { self.keyboard[0xE] = true; },
                        Keycode::Z => { self.keyboard[0xA] = true; },
                        Keycode::X => { self.keyboard[0x0] = true; },
                        Keycode::C => { self.keyboard[0xB] = true; },
                        Keycode::V => { self.keyboard[0xF] = true; },
                        _ => {}
                    }
                }
                Event::KeyUp {keycode, ..} => {
                    match keycode.unwrap() {
                        Keycode::Num1 => { self.keyboard[0x1] = false; },
                        Keycode::Num2 => { self.keyboard[0x2] = false; },
                        Keycode::Num3 => { self.keyboard[0x3] = false; },
                        Keycode::Num4 => { self.keyboard[0xC] = false; },
                        Keycode::Q => { self.keyboard[0x4] = false; },
                        Keycode::W => { self.keyboard[0x5] = false; },
                        Keycode::E => { self.keyboard[0x6] = false; },
                        Keycode::R => { self.keyboard[0xD] = false; },
                        Keycode::A => { self.keyboard[0x7] = false; },
                        Keycode::S => { self.keyboard[0x8] = false; },
                        Keycode::D => { self.keyboard[0x9] = false; },
                        Keycode::F => { self.keyboard[0xE] = false; },
                        Keycode::Z => { self.keyboard[0xA] = false; },
                        Keycode::X => { self.keyboard[0x0] = false; },
                        Keycode::C => { self.keyboard[0xB] = false; },
                        Keycode::V => { self.keyboard[0xF] = false; },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        return true;
    }

}