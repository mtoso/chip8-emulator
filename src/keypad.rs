// Keypad                   Keyboard
// +-+-+-+-+                +-+-+-+-+
// |1|2|3|C|                |1|2|3|4|
// +-+-+-+-+                +-+-+-+-+
// |4|5|6|D|                |Q|W|E|R|
// +-+-+-+-+       =>       +-+-+-+-+
// |7|8|9|E|                |A|S|D|F|
// +-+-+-+-+                +-+-+-+-+
// |A|0|B|F|                |Z|X|C|V|
// +-+-+-+-+                +-+-+-+-+

pub struct Keypad {
    pub keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Self {
        Keypad { keys: [false; 16] }
    }

    pub fn key_down(&mut self, index: u8) {
        self.keys[index as usize] = true;
    }

    pub fn key_up(&mut self, index: u8) {
        self.keys[index as usize] = false;
    }

    pub fn is_key_dow(&self, index: u8) -> bool {
        self.keys[index as usize]
    }
}
