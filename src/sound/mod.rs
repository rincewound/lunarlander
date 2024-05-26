use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;

pub struct Sound {
    stream_handle: OutputStreamHandle,
    stream: OutputStream,
    bg_music_sink: Sink,
}

impl Sound {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            stream_handle,
            stream: _stream,
            bg_music_sink: sink,
        }
    }

    pub fn shoot(&self) {
        let laser_file = BufReader::new(File::open("./assets/sfx_wpn_laser9.wav").unwrap());
        let shoot = Decoder::new(laser_file).unwrap();
        self.stream_handle
            .play_raw(shoot.convert_samples())
            .unwrap();
    }

    pub fn explode(&self) {
        let explosion_file =
            BufReader::new(File::open("./assets/sfx_exp_short_hard8.wav").unwrap());
        let explode = Decoder::new(explosion_file).unwrap();
        self.stream_handle
            .play_raw(explode.convert_samples())
            .unwrap();
    }

    pub fn accelerate(&self) {
        let acc_file = BufReader::new(File::open("./assets/sfx_vehicle_engineloop.wav").unwrap());
        let acc = Decoder::new(acc_file).unwrap();
        self.stream_handle.play_raw(acc.convert_samples()).unwrap();
    }

    pub fn play_background_music(&mut self) {
        let file = BufReader::new(File::open("./assets/bg_music.mp3").unwrap());
        self.bg_music_sink.append(Decoder::new(file).unwrap());
        self.bg_music_sink.play();
    }

    pub fn toggle_background_music(&mut self) {
        if self.bg_music_sink.is_paused() {
            self.bg_music_sink.play();
        } else {
            self.bg_music_sink.pause();
        }
    }

    pub fn die(&self) {
        let file = BufReader::new(File::open("./assets/sfx_deathscream_robot4.wav").unwrap());
        let dec = Decoder::new(file).unwrap();
        self.stream_handle.play_raw(dec.convert_samples()).unwrap();
    }
}
