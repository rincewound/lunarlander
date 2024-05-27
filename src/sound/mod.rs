use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

use std::convert::AsRef;
use std::io;
use std::sync::Arc;

pub struct Sfx(Arc<Vec<u8>>);

impl AsRef<[u8]> for Sfx {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sfx {
    pub fn load(filename: &str) -> io::Result<Sfx> {
        let mut buf = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut buf)?;
        Ok(Sfx(Arc::new(buf)))
    }
    pub fn cursor(self: &Self) -> io::Cursor<Sfx> {
        io::Cursor::new(Sfx(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sfx>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}

pub struct Sound {
    stream_handle: OutputStreamHandle,
    stream: OutputStream,
    bg_music_sink: Sink,
    samples: HashMap<String, Sfx>,
}

impl Sound {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        let mut samples: HashMap<String, Sfx> = HashMap::new();
        let files = [
            "./assets/Fire_normal.Mp3",
            "./assets/sfx_vehicle_engineloop.wav",
            "./assets/2.mp3",
            "./assets/Enemy_explode.mp3",
        ];

        for filename in files.iter() {
            if let Ok(sfxfile) = Sfx::load(filename) {
                samples.insert(filename.to_string(), sfxfile);
            }
        }

        Self {
            stream_handle,
            stream: _stream,
            bg_music_sink: sink,
            samples: samples,
        }
    }

    fn play_sample(&self, sample_name: &str) {
        if let Some(snd) = self.samples.get(sample_name) {
            let _ = self.stream_handle.play_raw(snd.decoder().convert_samples());
        }
    }

    pub fn shoot(&self) {
        self.play_sample("./assets/Fire_normal.mp3");
    }

    pub fn explode(&self) {
        self.play_sample("./assets/Enemy_explode.mp3");
    }

    pub fn accelerate(&self) {
        self.play_sample("./assets/sfx_vehicle_engineloop.wav")
    }

    pub fn play_background_music(&mut self) {
        if let Some(snd) = self.samples.get("./assets/2.mp3") {
            if self.bg_music_sink.len() < 10 {
                self.bg_music_sink.append(snd.decoder());
            }
        }
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
