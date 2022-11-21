extern crate sdl2;
use sdl2::{Sdl, audio::{AudioSpecDesired, AudioCallback, AudioDevice}};


pub struct AudioDriver {
    device: AudioDevice<SquareWave>
}

impl AudioDriver {
    pub fn new(sdl_context: &Sdl) -> AudioDriver {
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                SquareWave {
                    phase_inc: 240.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25
                }
            }).unwrap();

        device.resume();
        AudioDriver { device }
    }

    pub fn beep(&self, should_beep: bool) {
        if should_beep {
            self.device.resume();
        } else {
            self.device.pause();
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        // generate square wave
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
