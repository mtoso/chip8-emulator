// https://codereview.stackexchange.com/questions/169172/complementary-multiply-with-carry-in-rust/169338
pub const CMWC_CYCLE: usize = 4096;
const PHI: u32 = 0x9e3779b9;

pub struct ComplementaryMultiplyWithCarryGen {
    pub q: [u32; CMWC_CYCLE],
    pub c: u32,
    pub i: usize,
}

impl ComplementaryMultiplyWithCarryGen {
    pub fn new(seed: u32) -> ComplementaryMultiplyWithCarryGen {
        let mut q = [0; CMWC_CYCLE];

        q[0] = seed;
        q[1] = seed.wrapping_add(PHI);
        q[2] = seed.wrapping_add(PHI).wrapping_add(PHI);

        for i in 3..CMWC_CYCLE {
            let window = &mut q[i - 3..i + 1];
            window[3] = window[0] ^ window[1] ^ PHI ^ seed;
        }

        ComplementaryMultiplyWithCarryGen {
            q: q,
            c: 362436,
            i: 4095,
        }
    }

    pub fn random(&mut self) -> u32 {
        const A: u64 = 18782;
        const R: u32 = 0xfffffffe;

        self.i = (self.i + 1) & (CMWC_CYCLE - 1);
        let t = A * self.q[self.i] as u64 + self.c as u64;

        self.c = (t >> 32) as u32;
        let mut x = (t + self.c as u64) as u32;
        if x < self.c {
            x += 1;
            self.c += 1;
        }

        self.q[self.i] = R - x;
        self.q[self.i]
    }
}

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

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display {
    pub memory: [u8; 2048], // WIDTH * HEIGHT
}

impl Display {
    pub fn new() -> Self {
        Display { memory: [0; 2048] }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        self.memory[x + y * WIDTH] = on as u8;
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        self.memory[x + y * WIDTH] == 1
    }

    pub fn cls(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
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
                let value = row >> (7 - i) & 0x1;
                if value == 1 {
                    // calculate the indexes in the memory
                    let xi = (x + i) % WIDTH;
                    let yj = (y + j) % HEIGHT;
                    // get the value on the screen in order to detect collisions
                    let value_screen_on = self.get_pixel(xi, yj);
                    if value_screen_on {
                        collision = true;
                    }
                    // draw the new value with XOR
                    self.set_pixel(xi, yj, (value == 1) ^ value_screen_on);
                }
            }
        }
        return collision;
    }
}

pub static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Cpu {
    // index register
    pub i: u16,
    // program counter: from 0 to 0xFFF
    pub pc: u16,
    // memory: max 4096 = 2^(16-4) the 4-bit are used to identify the instruction
    pub memory: [u8; 4096],
    // registers: 15 8-bit general purpose V0...VE
    // The register VF is used as carry flag
    pub v: [u8; 16],
    // stack
    pub stack: [u16; 16],
    // stack pointer
    pub sp: u8,
    // delay timer
    pub dt: u8,
    // random number generator using CMWC algo
    pub rand: ComplementaryMultiplyWithCarryGen,
    // display
    pub display: Display,
    // keypad
    pub keypad: Keypad,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            i: 0,
            pc: 0, // TODO: should this start at 0x200?
            memory: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            rand: ComplementaryMultiplyWithCarryGen::new(1),
            display: Display::new(),
            keypad: Keypad::new(),
        }
    }

    pub fn execute_cycle(&mut self) {
        // read the opcode from the memory
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.process_opcode(opcode);
    }

    fn process_opcode(&mut self, opcode: u16) {
        // extract opcode parameters
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x]; // register
        let vy = self.v[y]; // register
        let nnn = opcode & 0x0FFF; // memory address
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        // break up the opcode into nibbles (4-bits)
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        // increment the program counter
        self.pc += 2;

        // process the opcode
        match (op_1, op_2, op_3, op_4) {
            // 00E0 - CLS
            // Clear the display.
            (0, 0, 0xE, 0) => self.display.cls(),

            // 00EE - RET
            // Return from a subroutine.
            (0, 0, 0xE, 0xE) => {
                self.sp = self.sp - 1;
                self.pc = self.stack[self.sp as usize];
            }

            // 1nnn - JP addr
            // Jump to location nnn.
            (0x1, _, _, _) => self.pc = nnn,

            // 2nnn - CALL addr
            // Call subroutine at nnn.
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }

            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx = kk.
            (0x3, _, _, _) => self.pc += if vx == kk { 2 } else { 0 },

            // 4xkk - SNE Vx, byte
            // Skip next instruction if Vx != kk.
            (0x4, _, _, _) => self.pc += if vx != kk { 2 } else { 0 },

            // 5xy0 - SE Vx, Vy
            // Skip next instruction if Vx = Vy.
            (0x5, _, _, 0) => self.pc += if vx == vy { 2 } else { 0 },

            // 6xkk - LD Vx, byte
            // Set Vx = kk.
            (0x6, _, _, _) => self.v[x] = kk,

            // 7xkk - ADD Vx, byte
            // Set Vx = Vx + kk
            (0x7, _, _, _) => self.v[x] = vx + kk,

            // 8xy0 - LD Vx, Vy
            // Set Vx = Vy.
            (0x8, _, _, 0) => self.v[x] = vy,

            // 8xy1 - OR Vx, Vy
            // Set Vx = Vx OR Vy.
            (0x8, _, _, 0x1) => self.v[x] = vx | vy,

            // 8xy2 - AND Vx, Vy
            // Set Vx = Vx AND Vy
            (0x8, _, _, 0x2) => self.v[x] = vx & vy,

            // 8xy3 - XOR Vx, Vy
            // Set Vx = Vx XOR Vy.
            (0x8, _, _, 0x3) => self.v[x] = vx ^ vy,

            // 8xy4 - ADD Vx, Vy
            // Set Vx = Vx + Vy, set VF = carry.
            (0x8, _, _, 0x4) => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                match overflow {
                    true => self.v[0xF] = 1,
                    false => self.v[0xF] = 0,
                }
                self.v[x] = res;
            }

            // 8xy5 - SUB Vx, Vy
            // Set Vx = Vx - Vy, set VF = NOT borrow.
            (0x8, _, _, 0x5) => {
                self.v[0xF] = if vx > vy { 1 } else { 0 };
                self.v[x] = vx - vy;
            }

            // 8xy6 - SHR Vx {, Vy}
            // Set Vx = Vx SHR 1.
            (0x8, _, _, 0x6) => {
                self.v[0xF] = vx & 0x1;
                self.v[x] >>= 1;
            }

            // 8xy7 - SUBN Vx, Vy
            // Set Vx = Vy - Vx, set VF = NOT borrow.
            (0x8, _, _, 0x7) => {
                self.v[0xF] = if vy > vx { 1 } else { 0 };
                self.v[x] = vy - vx;
            }

            // 8xyE - SHL Vx {, Vy}
            // Set Vx = Vx SHL 1.
            (0x8, _, _, 0xE) => {
                self.v[0xF] = vx & 0x8;
                self.v[x] <<= 1;
            }

            // 9xy0 - SNE Vx, Vy
            // Skip next instruction if Vx != Vy.
            (0x9, _, _, 0) => self.pc += if vx != vy { 2 } else { 0 },

            // Annn - LD I, addr
            // Set I = nnn
            (0xA, _, _, _) => self.i = nnn,

            // Bnnn - JP V0, addr
            // Jump to location nnn + V0
            (0xB, _, _, _) => self.pc = nnn + self.v[0] as u16,

            // Cxkk - RND Vx, byte
            // Set Vx = random byte AND kk
            (0xC, _, _, _) => self.v[x] = self.rand.random() as u8 & kk,

            // Dxyn - DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            (0xD, _, _, _) => {
                let collision = self.display.draw(
                    vx as usize,
                    vy as usize,
                    &self.memory[self.i as usize..(self.i + n as u16) as usize],
                );
                self.v[0xF] = if collision { 1 } else { 0 };
            }

            // Ex9E - SKP Vx
            // Skip next instruction if key with the value of Vx is pressed
            (0xE, _, 0x9, 0xE) => self.pc += if self.keypad.is_key_dow(vx) { 2 } else { 0 },

            // ExA1 - SKNP Vx
            // Skip next instruction if key with the value of Vx is not pressed
            (0xE, _, 0xA, 0x1) => self.pc += if self.keypad.is_key_dow(vx) { 0 } else { 8 },

            // Fx07 - LD Vx, DT
            // Set Vx = delay timer value
            (0xF, _, 0x0, 0x7) => self.v[x] = self.dt,

            // Fx0A - LD Vx, K
            // Wait for a key press, store the value of the key in Vx
            (0xF, _, 0x0, 0xA) => {
                self.pc -= 2;
                for (i, key) in self.keypad.keys.iter().enumerate() {
                    if *key == true {
                        self.v[x] = i as u8;
                        self.pc += 2;
                    }
                }
            }

            // Fx15 - LD DT, Vx
            // Set delay timer = Vx
            (0xF, _, 0x1, 0x5) => self.dt = vx,

            // Fx1E - ADD I, Vx
            // Set I = I + Vx
            (0xF, _, 1, 0xE) => self.i = self.i + vx as u16,

            // Fx29 - LD F, Vx
            // Set I = location of sprite for digit Vx
            (0xF, _, 0x2, 0x9) => self.i = vx as u16 * 5,

            // Fx33 - LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1, and I+2
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = (vx % 100) % 10;
            }

            // Fx55 - LD [I], Vx
            // Store registers V0 through Vx in memory starting at location I
            (0xF, _, 0x5, 0x5) => self.memory[(self.i as usize)..(self.i + x as u16 + 1) as usize]
                .copy_from_slice(&self.v[0..(x as usize + 1)]),

            // Fx65 - LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I
            (0xF, _, 0x6, 0x5) => self.v[0..(x as usize + 1)]
                .copy_from_slice(&self.memory[(self.i as usize)..(self.i + x as u16 + 1) as usize]),

            (_, _, _, _) => unimplemented!("opcode {:04x}", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use super::Display;

    #[test]
    fn opcode_jp() {
        let mut cpu = Cpu::new();
        cpu.process_opcode(0x1A2A);
        assert_eq!(cpu.pc, 0x0A2A, "the program counter is updated");
    }

    #[test]
    fn opcode_call() {
        let mut cpu = Cpu::new();
        let addr = 0x23;
        cpu.pc = addr;

        cpu.process_opcode(0x2ABC);

        assert_eq!(
            cpu.pc, 0x0ABC,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.sp, 1, "the stack pointer is incremented");
        assert_eq!(
            cpu.stack[0],
            addr + 2,
            "the stack stores the previous address"
        );
    }

    #[test]
    fn opcode_se_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 0xFE;
        // vx == kk
        cpu.process_opcode(0x31FE);
        assert_eq!(cpu.pc, 4, "the stack pointer skips");

        // vx != kk
        cpu.process_opcode(0x31FA);
        assert_eq!(cpu.pc, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 0xFE;
        // vx == kk
        cpu.process_opcode(0x41FE);
        assert_eq!(cpu.pc, 2, "the stack pointer is incremented");

        // vx != kk
        cpu.process_opcode(0x41FA);
        assert_eq!(cpu.pc, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_se_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 1;
        cpu.v[2] = 3;
        cpu.v[3] = 3;
        // vx == vy
        cpu.process_opcode(0x5230);
        assert_eq!(cpu.pc, 4, "the stack pointer skips");

        // vx != vy
        cpu.process_opcode(0x5130);
        assert_eq!(cpu.pc, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 1;
        cpu.v[2] = 3;
        cpu.v[3] = 3;
        // vx == vy
        cpu.process_opcode(0x9230);
        assert_eq!(cpu.pc, 2, "the stack pointer is incremented");

        // vx != vy
        cpu.process_opcode(0x9130);
        assert_eq!(cpu.pc, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_add_vx_kkk() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 3;
        cpu.process_opcode(0x7101);
        assert_eq!(cpu.v[1], 4, "Vx was incremented by one");
    }

    #[test]
    fn opcode_ld_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 3;
        cpu.v[0] = 0;
        cpu.process_opcode(0x8010);
        assert_eq!(cpu.v[0], 3, "Vx was loaded with vy");
    }

    #[test]
    fn opcode_or_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.process_opcode(0x8231);
        assert_eq!(cpu.v[2], 0b11101110, "Vx was loaded with vx OR vy");
    }

    #[test]
    fn opcode_and_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.process_opcode(0x8232);
        assert_eq!(cpu.v[2], 0b01001100, "Vx was loaded with vx AND vy");
    }

    #[test]
    fn opcode_xor_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.process_opcode(0x8233);
        assert_eq!(cpu.v[2], 0b10100010, "Vx was loaded with vx XOR vy");
    }

    #[test]
    fn opcode_add_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 10;
        cpu.v[2] = 100;
        cpu.v[3] = 250;
        cpu.process_opcode(0x8124);
        assert_eq!(cpu.v[1], 110, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v[0xF], 0, "no overflow occured");

        cpu.process_opcode(0x8134);
        assert_eq!(cpu.v[1], 0x68, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v[0xF], 1, "overflow occured");
    }

    #[test]
    fn opcode_ld_i_vx() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 5;
        cpu.v[1] = 4;
        cpu.v[2] = 3;
        cpu.v[3] = 2;
        cpu.i = 0x300;
        // load v0 - v2 into memory at i
        cpu.process_opcode(0xF255);
        assert_eq!(
            cpu.memory[cpu.i as usize], 5,
            "V0 was loaded into memory at i"
        );
        assert_eq!(
            cpu.memory[cpu.i as usize + 1],
            4,
            "V1 was loaded into memory at i + 1"
        );
        assert_eq!(
            cpu.memory[cpu.i as usize + 2],
            3,
            "V2 was loaded into memory at i + 2"
        );
        assert_eq!(cpu.memory[cpu.i as usize + 3], 0, "i + 3 was not loaded");
    }
    #[test]
    fn opcode_ld_b_vx() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.v[2] = 234;
        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF233);
        assert_eq!(cpu.memory[cpu.i as usize], 2, "hundreds");
        assert_eq!(cpu.memory[cpu.i as usize + 1], 3, "tens");
        assert_eq!(cpu.memory[cpu.i as usize + 2], 4, "digits");
    }
    #[test]
    fn opcode_ld_vx_i() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.memory[cpu.i as usize] = 5;
        cpu.memory[cpu.i as usize + 1] = 4;
        cpu.memory[cpu.i as usize + 2] = 3;
        cpu.memory[cpu.i as usize + 3] = 2;
        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF265);
        assert_eq!(cpu.v[0], 5, "V0 was loaded from memory at i");
        assert_eq!(cpu.v[1], 4, "V1 was loaded from memory at i + 1");
        assert_eq!(cpu.v[2], 3, "V2 was loaded from memory at i + 2");
        assert_eq!(cpu.v[3], 0, "i + 3 was not loaded");
    }

    #[test]
    fn opcode_ret() {
        let mut cpu = Cpu::new();
        let addr = 0x23;
        cpu.pc = addr;

        // jump to 0x0ABC
        cpu.process_opcode(0x2ABC);
        // return
        cpu.process_opcode(0x00EE);

        assert_eq!(
            cpu.pc, 0x25,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.sp, 0, "the stack pointer is decremented");
    }
    #[test]
    fn opcode_ld_i_addr() {
        let mut cpu = Cpu::new();

        cpu.process_opcode(0x61AA);
        assert_eq!(cpu.v[1], 0xAA, "V1 is set");
        assert_eq!(cpu.pc, 2, "the program counter is advanced two bytes");

        cpu.process_opcode(0x621A);
        assert_eq!(cpu.v[2], 0x1A, "V2 is set");
        assert_eq!(cpu.pc, 4, "the program counter is advanced two bytes");

        cpu.process_opcode(0x6A15);
        assert_eq!(cpu.v[10], 0x15, "V10 is set");
        assert_eq!(cpu.pc, 6, "the program counter is advanced two bytes");
    }

    #[test]
    fn opcode_axxx() {
        let mut cpu = Cpu::new();
        cpu.process_opcode(0xAFAF);

        assert_eq!(cpu.i, 0x0FAF, "the 'i' register is updated");
        assert_eq!(cpu.pc, 2, "the program counter is advanced two bytes");
    }

    #[test]
    fn set_pixel() {
        let mut display = Display::new();
        display.set_pixel(1, 1, true);
        assert_eq!(true, display.get_pixel(1, 1));
    }

    #[test]
    fn cls() {
        let mut display = Display::new();
        display.set_pixel(1, 1, true);
        display.cls();
        assert_eq!(false, display.get_pixel(1, 1));
    }

    #[test]
    fn draw() {
        let mut display = Display::new();
        let sprite: [u8; 2] = [0b00110011, 0b11001010];
        display.draw(0, 0, &sprite);

        assert_eq!(false, display.get_pixel(0, 0));
        assert_eq!(false, display.get_pixel(1, 0));
        assert_eq!(true, display.get_pixel(2, 0));
        assert_eq!(true, display.get_pixel(3, 0));
        assert_eq!(false, display.get_pixel(4, 0));
        assert_eq!(false, display.get_pixel(5, 0));
        assert_eq!(true, display.get_pixel(6, 0));
        assert_eq!(true, display.get_pixel(7, 0));

        assert_eq!(true, display.get_pixel(0, 1));
        assert_eq!(true, display.get_pixel(1, 1));
        assert_eq!(false, display.get_pixel(2, 1));
        assert_eq!(false, display.get_pixel(3, 1));
        assert_eq!(true, display.get_pixel(4, 1));
        assert_eq!(false, display.get_pixel(5, 1));
        assert_eq!(true, display.get_pixel(6, 1));
        assert_eq!(false, display.get_pixel(7, 1));
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
