const MEMORY_SIZE: usize = 4096;
const DISPLAY_PIXEL_WIDTH: usize = 64;
const DISPLAY_PIXEL_HEIGHT: usize = 32;

pub mod cpu;
pub mod display;
pub mod keypad;
pub mod rand;
pub mod font;
pub mod cartridge;