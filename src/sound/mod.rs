use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

pub struct Sound {
    stream_handle: OutputStreamHandle,
    stream: OutputStream,
}

impl Sound {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            stream_handle,
            stream: _stream,
        }
    }

    pub fn shoot(&self) {
        let laser_file = BufReader::new(File::open("./assets/sfx_wpn_laser9.wav").unwrap());
        let shoot = Decoder::new(laser_file).unwrap();
        self.stream_handle.play_raw(shoot.convert_samples()).unwrap();
    }

    pub fn explode(&self) {
        let explosion_file = BufReader::new(File::open("./assets/sfx_exp_short_hard8.wav").unwrap());
        let explode = Decoder::new(explosion_file).unwrap();
        self.stream_handle.play_raw(explode.convert_samples()).unwrap();
    }
}
