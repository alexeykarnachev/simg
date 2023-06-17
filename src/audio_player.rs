use sdl2::mixer::{InitFlag, Music, AUDIO_S16LSB, DEFAULT_CHANNELS};

pub struct AudioPlayer<'a> {
    pub is_initialized: bool,
    musics: Vec<Music<'a>>,
}

impl AudioPlayer<'_> {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            musics: vec![],
        }
    }

    pub fn init(&mut self, sdl2: &sdl2::Sdl) {
        let _audio = sdl2.audio().unwrap();
        Box::leak(Box::new(_audio));

        let format = AUDIO_S16LSB;
        let n_channels = DEFAULT_CHANNELS;
        let frequency = 44_100;
        let chunk_size = 1_024;
        let n_mixed_channels = 4;

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

    pub fn play_music(&mut self, idx: usize) {
        let music = &mut self.musics[idx];
        music.play(-1).unwrap();
    }
}
