use sdl2::mixer::{Channel, Chunk, InitFlag, Music, AUDIO_S16SYS};

pub struct AudioPlayer<'a> {
    pub is_initialized: bool,
    musics: Vec<Music<'a>>,
    chunks: Vec<Chunk>,
}

impl AudioPlayer<'_> {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            musics: vec![],
            chunks: vec![],
        }
    }

    pub fn init(&mut self, sdl2: &sdl2::Sdl) {
        let _audio = sdl2.audio().unwrap();
        Box::leak(Box::new(_audio));

        let format = AUDIO_S16SYS;
        let n_channels = 2;
        let frequency = 44100;
        let chunk_size = 1024;
        let n_mixed_channels = 8;

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

    pub fn load_chunk_from_wav_bytes(&mut self, bytes: &[u8]) -> usize {
        let rwops = sdl2::rwops::RWops::from_bytes(bytes).unwrap();
        let chunk = sdl2::mixer::LoaderRWops::load_wav(&rwops).unwrap();
        self.chunks.push(chunk);

        self.chunks.len() - 1
    }

    pub fn load_chunk_from_file(&mut self, file_path: &str) -> usize {
        let sound = Chunk::from_file(file_path).unwrap();
        self.chunks.push(sound);

        self.chunks.len() - 1
    }

    pub fn play_music(&mut self, idx: usize) {
        let music = &mut self.musics[idx];
        music.play(-1).unwrap();
    }

    pub fn fade_out_music(&self, ms: u32) {
        Music::fade_out(ms as i32).unwrap();
    }

    pub fn play_chunk(&self, idx: usize) {
        let sound = &self.chunks[idx];
        let _ = Channel::all().play(sound, 0);
    }
}
