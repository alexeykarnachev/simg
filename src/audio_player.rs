use sdl2::mixer::{
    Channel, Chunk, InitFlag, Music, AUDIO_S16LSB, DEFAULT_CHANNELS,
};

pub struct AudioPlayer<'a> {
    pub is_initialized: bool,
    musics: Vec<Music<'a>>,
    sounds: Vec<Chunk>,
}

impl AudioPlayer<'_> {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            musics: vec![],
            sounds: vec![],
        }
    }

    pub fn init(&mut self, sdl2: &sdl2::Sdl) {
        let _audio = sdl2.audio().unwrap();
        Box::leak(Box::new(_audio));

        let format = AUDIO_S16LSB;
        let n_channels = DEFAULT_CHANNELS;
        let frequency = 48000;
        let chunk_size = 256;
        let n_mixed_channels = 16;

        sdl2::mixer::open_audio(frequency, format, n_channels, chunk_size)
            .unwrap();
        let _mixer_context = sdl2::mixer::init(
            InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG,
        )
        .unwrap();
        sdl2::mixer::allocate_channels(n_mixed_channels);

        self.is_initialized = true;
    }

    pub fn load_music_from_bytes(
        &mut self,
        bytes: &'static [u8],
    ) -> usize {
        let music = Music::from_static_bytes(bytes).unwrap();
        let idx = self.musics.len();
        self.musics.push(music);

        idx
    }

    pub fn load_sound_from_bytes(&mut self, bytes: &[i16]) -> usize {
        let buffer = bytes.to_vec().into_boxed_slice();
        let sound = Chunk::from_raw_buffer(buffer).unwrap();
        let idx = self.sounds.len();
        self.sounds.push(sound);

        idx
    }

    pub fn load_sound_from_file(&mut self, file_path: &str) -> usize {
        let sound = Chunk::from_file(file_path).unwrap();
        let idx = self.sounds.len();
        self.sounds.push(sound);

        idx
    }

    pub fn play_music(&mut self, idx: usize) {
        let music = &mut self.musics[idx];
        music.play(-1).unwrap();
    }

    pub fn play_sound(&self, idx: usize) {
        let sound = &self.sounds[idx];
        let _ = Channel::all().play(sound, 0);
    }
}
