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
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Self {
        Keypad { keys: [false; 16] }
    }

    fn key_to_idx(&self, key: &str) -> Option<usize> {
        let idx = match key.to_lowercase().as_ref() {
            "1" => Some(0),
            "2" => Some(1),
            "3" => Some(2),
            "4" => Some(3),
            "q" => Some(4),
            "w" => Some(5),
            "e" => Some(6),
            "r" => Some(7),
            "a" => Some(8),
            "s" => Some(9),
            "d" => Some(10),
            "f" => Some(11),
            "z" => Some(12),
            "x" => Some(13),
            "c" => Some(14),
            "v" => Some(15),
            _ => None,
        };

        idx
    }

    pub fn key_down(&mut self, key: &str) {
        match self.key_to_idx(key) {
            Some(idx) => self.keys[idx] = true,
            None => (),
        }
    }

    pub fn key_up(&mut self, key: &str) {
        match self.key_to_idx(key) {
            Some(idx) => self.keys[idx] = false,
            None => (),
        }
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        let is_pressed = match self.key_to_idx(key) {
            Some(idx) => self.is_key_idx_pressed(idx),
            None => false,
        };

        is_pressed
    }
    pub fn is_key_idx_pressed(&self, idx: usize) -> bool {
        self.keys[idx] == true
    }
    pub fn get_first_pressed_key_idx(&self) -> Option<usize> {
        self.keys.iter().position(|&idx| idx == true)
    }
}

#[cfg(test)]
mod tests {
    use super::Keypad;

    #[test]
    fn it_sets_key() {
        let mut keypad = Keypad::new();

        keypad.key_down("a");
        assert!(keypad.is_key_pressed("a"));
        assert!(keypad.is_key_idx_pressed(8));
    }
    #[test]
    fn it_unsets_key() {
        let mut keypad = Keypad::new();

        keypad.key_down("a");
        keypad.key_up("a");
        assert!(!keypad.is_key_pressed("a"));
        assert!(!keypad.is_key_idx_pressed(8));
    }

    #[test]
    fn default_key_state_is_off() {
        let keypad = Keypad::new();
        assert!(!keypad.is_key_pressed("a"));
    }

    #[test]
    fn it_returns_first_pressed_idx() {
        let mut keypad = Keypad::new();

        keypad.key_down("2");
        keypad.key_down("f");

        assert_eq!(keypad.get_first_pressed_key_idx(), Some(1));
    }

    #[test]
    fn it_returns_none_if_no_key_pressed() {
        let keypad = Keypad::new();

        assert_eq!(keypad.get_first_pressed_key_idx(), None);
    }
}
