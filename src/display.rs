use super::DISPLAY_PIXEL_HEIGHT;
use super::DISPLAY_PIXEL_WIDTH;

const VRAM_SIZE: usize = DISPLAY_PIXEL_WIDTH * DISPLAY_PIXEL_HEIGHT;

pub struct Display {
    vram: [u8; VRAM_SIZE],
}

impl Display {
    pub fn new() -> Self {
        Display { vram: [0; 2048] }
    }

    fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        let scaled = y * DISPLAY_PIXEL_WIDTH;
        self.vram[x + scaled] = on as u8;
    }

    fn is_pixel_on(&mut self, x: usize, y: usize) -> bool {
        let scaled = y * DISPLAY_PIXEL_WIDTH;
        self.vram[x + scaled] == 1
    }

    pub fn cls(&mut self) {
        for x in 0..DISPLAY_PIXEL_WIDTH {
            for y in 0..DISPLAY_PIXEL_HEIGHT {
                self.set_pixel(x, y, false);
            }
        }
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        for j in 0..rows {
            let row = sprite[j];
            for i in 0..8 {
                // check every single bit, starting from the most significant bit
                let value = row >> (7 - i) & 0x01;
                if value == 1 {
                    // calculate the indexes in the memory
                    let xi = (x + i) % DISPLAY_PIXEL_WIDTH;
                    let yj = (y + j) % DISPLAY_PIXEL_HEIGHT;
                    // get the value on the screen in order to detect collisions
                    let value_screen_on = self.is_pixel_on(xi, yj);
                    if value_screen_on {
                        collision = true;
                    }
                    // draw the new value with XOR
                    let is_on = (value == 1) ^ value_screen_on;
                    self.set_pixel(xi, yj, is_on);
                }
            }
        }
        return collision;
    }

    pub fn get_vram_copy(&self) -> Vec<u8> {
        self.vram.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::Display;

    #[test]
    fn set_pixel() {
        let mut display = Display::new();
        display.set_pixel(1, 1, true);
        assert_eq!(true, display.is_pixel_on(1, 1));
    }

    #[test]
    fn cls() {
        let mut display = Display::new();
        display.set_pixel(1, 1, true);
        display.cls();
        assert_eq!(false, display.is_pixel_on(1, 1));
    }

    #[test]
    fn draw() {
        let mut display = Display::new();
        let sprite: [u8; 2] = [0b00110011, 0b11001010];
        display.draw(0, 0, &sprite);

        assert_eq!(false, display.is_pixel_on(0, 0));
        assert_eq!(false, display.is_pixel_on(1, 0));
        assert_eq!(true, display.is_pixel_on(2, 0));
        assert_eq!(true, display.is_pixel_on(3, 0));
        assert_eq!(false, display.is_pixel_on(4, 0));
        assert_eq!(false, display.is_pixel_on(5, 0));
        assert_eq!(true, display.is_pixel_on(6, 0));
        assert_eq!(true, display.is_pixel_on(7, 0));

        assert_eq!(true, display.is_pixel_on(0, 1));
        assert_eq!(true, display.is_pixel_on(1, 1));
        assert_eq!(false, display.is_pixel_on(2, 1));
        assert_eq!(false, display.is_pixel_on(3, 1));
        assert_eq!(true, display.is_pixel_on(4, 1));
        assert_eq!(false, display.is_pixel_on(5, 1));
        assert_eq!(true, display.is_pixel_on(6, 1));
        assert_eq!(false, display.is_pixel_on(7, 1));
    }

    #[test]
    fn draw_detects_collisions() {
        let mut display = Display::new();

        let mut sprite: [u8; 1] = [0b00110000];
        let mut collision = display.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000011];
        collision = display.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000001];
        collision = display.draw(0, 0, &sprite);
        assert_eq!(true, collision);
    }
}
