#![allow(unused_variables)]
#![allow(dead_code)]
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::MouseButton;
use std::collections::HashSet;
use std::hash::Hash;

const KEYBOARD_KEYS_QUEUE_LEN: usize = 16;
const N_KEYBOARD_KEYS: usize = 1024;

pub struct KeyStates<T: Copy + Eq + Hash> {
    pub pressed: HashSet<T>,
    pub just_pressed: HashSet<T>,
    pub just_released: HashSet<T>,
}

impl<T> KeyStates<T>
where
    T: Copy + Eq + Hash,
{
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn press(&mut self, key: T) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub fn release(&mut self, key: T) {
        if self.pressed.remove(&key) {
            self.just_released.remove(&key);
        }
    }
}

impl<T> Default for KeyStates<T>
where
    T: Copy + Eq + Hash,
{
    fn default() -> Self {
        Self {
            pressed: HashSet::default(),
            just_pressed: HashSet::default(),
            just_released: HashSet::default(),
        }
    }
}

pub struct Input {
    event_pump: &'static mut sdl2::EventPump,

    pub should_quit: bool,
    pub keycodes: KeyStates<Keycode>,
    pub scancodes: KeyStates<Scancode>,
    pub mouse_buttons: KeyStates<MouseButton>,
}

impl Input {
    pub fn new(sdl2: &sdl2::Sdl) -> Self {
        let event_pump = Box::leak(Box::new(sdl2.event_pump().unwrap()));

        Self {
            event_pump,
            should_quit: false,
            keycodes: Default::default(),
            scancodes: Default::default(),
            mouse_buttons: Default::default(),
        }
    }

    pub fn update(&mut self) {
        self.keycodes.clear();
        self.scancodes.clear();
        self.mouse_buttons.clear();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.should_quit = true,
                Event::KeyDown {
                    keycode: Some(keycode),
                    scancode: Some(scancode),
                    ..
                } => {
                    self.keycodes.press(keycode);
                    self.scancodes.press(scancode);
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    scancode: Some(scancode),
                    ..
                } => {
                    self.keycodes.release(keycode);
                    self.scancodes.release(scancode);
                }
                Event::MouseButtonDown { mouse_btn, .. } => {
                    self.mouse_buttons.press(mouse_btn);
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    self.mouse_buttons.release(mouse_btn);
                }
                Event::TextInput { text, .. } => {}
                _ => {}
            }
        }
    }
}