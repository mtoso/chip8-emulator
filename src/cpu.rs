use wasm_bindgen::prelude::*;

use super::cartridge::Cartridge;
use super::display::Display;
use super::font::FONT_SET;
use super::keypad::Keypad;
use super::rand::ComplementaryMultiplyWithCarryGen;

use super::MEMORY_SIZE;

#[wasm_bindgen]
pub struct ExecutionResult {
    display_state: Vec<u8>,
    should_beep: bool,
}

#[wasm_bindgen]
impl ExecutionResult {
    pub fn new(display_state: Vec<u8>, should_beep: bool) -> ExecutionResult {
        ExecutionResult {
            display_state,
            should_beep,
        }
    }

    pub fn get_display_state(&self) -> Vec<u8> {
        self.display_state.clone()
    }

    pub fn get_should_beep(&self) -> bool {
        self.should_beep
    }
}

#[wasm_bindgen]
pub struct Cpu {
    // index register
    i: u16,
    // program counter: from 0x200 to 0xFFF
    pc: u16,
    // memory: MEMORY_SIZE max 4096 = 2^(16-4) the 4-bit are used to identify the instruction
    memory: [u8; MEMORY_SIZE],
    // registers: 15 8-bit general purpose V0...VE
    // The register VF is used as carry flag
    v: [u8; 16],
    // stack
    stack: [u16; 16],
    // stack pointer
    sp: u8,
    // delay timer
    dt: u8,
    // sound timer
    st: u8,
    // random number generator using CMWC algo
    rand: ComplementaryMultiplyWithCarryGen,
    // display
    display: Display,
    // keypad
    keypad: Keypad,
}

#[wasm_bindgen]
impl Cpu {
    pub fn new() -> Cpu {
        // init the memory space: first we init the fonts
        let mut memory = [0u8; MEMORY_SIZE];
        memory[0..FONT_SET.len()].copy_from_slice(&FONT_SET);

        Cpu {
            i: 0,
            pc: 0x200,
            memory,
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            rand: ComplementaryMultiplyWithCarryGen::new(1),
            display: Display::new(),
            keypad: Keypad::new(),
        }
    }

    pub fn load_cartridge(&mut self, program: Cartridge) {
        let program_memory = program.get_memory();
        // init the memory with the program starting at the addr 0x200
        self.memory[0x200..0x200 + program_memory.len()].copy_from_slice(&program_memory);
    }

    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = 0x200;
        self.memory = [0u8; MEMORY_SIZE];
        self.memory[0..FONT_SET.len()].copy_from_slice(&FONT_SET);
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
        self.rand = ComplementaryMultiplyWithCarryGen::new(1);
        self.display.cls();
    }

    pub fn keypad_down(&mut self, key: &str) {
        self.keypad.key_down(key)
    }

    pub fn keypad_up(&mut self, key: &str) {
        self.keypad.key_up(key)
    }

    pub fn execute_cycle(&mut self) -> ExecutionResult {
        // read the opcode from the memory
        let opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.process_opcode(opcode);
        ExecutionResult::new(self.display.get_vram_copy(), self.st > 0)
    }

    fn update_timers(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }
        if self.dt > 0 {
            self.dt -= 1;
        }
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

        // update timers
        self.update_timers();

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
                // the pc is already beign incremented to the next instruction
                // so we save the current value
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
                let total = self.v[x] as u16 + self.v[y] as u16;
                self.v[0xF] = if total > 0xFF { 1 } else { 0 };
                self.v[x] = total as u8;
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
            (0xE, _, 0x9, 0xE) => self.pc += if self.keypad.is_key_idx_pressed(vx as usize) { 2 } else { 0 },

            // ExA1 - SKNP Vx
            // Skip next instruction if key with the value of Vx is not pressed
            (0xE, _, 0xA, 0x1) => self.pc += if self.keypad.is_key_idx_pressed(vx as usize) { 0 } else { 2 },

            // Fx07 - LD Vx, DT
            // Set Vx = delay timer value
            (0xF, _, 0x0, 0x7) => self.v[x] = self.dt,

            // Fx0A - LD Vx, K
            // Wait for a key press, store the value of the key in Vx
            (0xF, _, 0x0, 0xA) => {
                match self.keypad.get_first_pressed_key_idx() {
                    Some(idx) => {
                        self.v[x] = idx as u8;
                        self.pc += 2;
                    }
                    None => ()
                }
            }

            // Fx15 - LD DT, Vx
            // Set delay timer = Vx
            (0xF, _, 0x1, 0x5) => self.dt = vx,

            // Fx18 - LD ST, Vx
            // Set sound timer = Vx
            (0xF, _, 0x1, 0x8) => self.st = vx,

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
    use super::*;
    use Cartridge;

    #[test]
    fn opcode_jp() {
        let mut cpu = Cpu::new();
        cpu.load_cartridge(Cartridge::new(&[0x1A, 0x2A]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x0A2A, "the program counter is updated");
    }

    #[test]
    fn opcode_call() {
        let mut cpu = Cpu::new();
        cpu.load_cartridge(Cartridge::new(&[0x2A, 0xBC]));
        cpu.execute_cycle();
        assert_eq!(
            cpu.pc, 0x0ABC,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.sp, 1, "the stack pointer is incremented");
        assert_eq!(cpu.stack[0], 0x202, "the stack stores the previous address");
    }

    #[test]
    fn opcode_se_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 0xFE; // kk

        // vx == kk -> 0x31FE
        cpu.load_cartridge(Cartridge::new(&[0x31, 0xFE]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x204, "the stack pointer skips");

        cpu.reset();

        // vx != kk -> 0x31FA
        cpu.load_cartridge(Cartridge::new(&[0x31, 0xFA]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x202, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 0xFE;

        // vx == kk
        cpu.load_cartridge(Cartridge::new(&[0x41, 0xFE]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x202, "the stack pointer is incremented");

        cpu.reset();

        // vx != kk
        cpu.load_cartridge(Cartridge::new(&[0x41, 0xFA]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x204, "the stack pointer skips");
    }

    #[test]
    fn opcode_se_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 3;
        cpu.v[3] = 3;

        // vx == vy
        cpu.load_cartridge(Cartridge::new(&[0x52, 0x30]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x204, "the stack pointer skips");

        cpu.reset();
        cpu.v[1] = 1;
        cpu.v[3] = 3;

        // vx != vy
        cpu.load_cartridge(Cartridge::new(&[0x51, 0x30]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x202, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 3;
        cpu.v[3] = 3;

        // vx == vy
        cpu.load_cartridge(Cartridge::new(&[0x92, 0x30]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x202, "the stack pointer is incremented");

        cpu.reset();
        cpu.v[1] = 1;
        cpu.v[3] = 3;

        // vx != vy
        cpu.load_cartridge(Cartridge::new(&[0x91, 0x30]));
        cpu.execute_cycle();
        assert_eq!(cpu.pc, 0x204, "the stack pointer skips");
    }

    #[test]
    fn opcode_add_vx_kkk() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 3;
        cpu.load_cartridge(Cartridge::new(&[0x71, 0x01]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[1], 4, "Vx was incremented by one");
    }

    #[test]
    fn opcode_ld_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 3;
        cpu.v[0] = 0;
        cpu.load_cartridge(Cartridge::new(&[0x80, 0x10]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[0], 3, "Vx was loaded with vy");
    }

    #[test]
    fn opcode_or_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.load_cartridge(Cartridge::new(&[0x82, 0x31]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[2], 0b11101110, "Vx was loaded with vx OR vy");
    }

    #[test]
    fn opcode_and_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.load_cartridge(Cartridge::new(&[0x82, 0x32]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[2], 0b01001100, "Vx was loaded with vx AND vy");
    }

    #[test]
    fn opcode_xor_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;
        cpu.load_cartridge(Cartridge::new(&[0x82, 0x33]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[2], 0b10100010, "Vx was loaded with vx XOR vy");
    }

    #[test]
    fn opcode_add_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[1] = 10;
        cpu.v[2] = 100;

        cpu.load_cartridge(Cartridge::new(&[0x81, 0x24]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[1], 110, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v[0xF], 0, "no overflow occured");

        cpu.reset();
        cpu.v[1] = 10;
        cpu.v[3] = 250;

        cpu.load_cartridge(Cartridge::new(&[0x81, 0x34]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[1], 4, "Vx was loaded with vx + vy");
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
        cpu.load_cartridge(Cartridge::new(&[0xF2, 0x55]));
        cpu.execute_cycle();
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
        cpu.load_cartridge(Cartridge::new(&[0xF2, 0x33]));
        cpu.execute_cycle();
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

        // jump to 0x0ABC
        cpu.load_cartridge(Cartridge::new(&[0x2A, 0xBC]));

        cpu.execute_cycle();
        assert_eq!(
            cpu.pc, 0xABC,
            "the program counter is updated to the new address"
        );
        // return
        cpu.memory[0xABC..0xABC + 2].copy_from_slice(&[0x00, 0xEE]);
        cpu.execute_cycle();
        assert_eq!(cpu.sp, 0, "the stack pointer is decremented");
    }

    #[test]
    fn opcode_ld_i_addr() {
        let mut cpu = Cpu::new();

        cpu.load_cartridge(Cartridge::new(&[0x61, 0xAA]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[1], 0xAA, "V1 is set");
        assert_eq!(cpu.pc, 0x202, "the program counter is advanced two bytes");

        cpu.reset();
        cpu.load_cartridge(Cartridge::new(&[0x62, 0x1A]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[2], 0x1A, "V2 is set");
        assert_eq!(cpu.pc, 0x202, "the program counter is advanced two bytes");

        cpu.reset();
        cpu.load_cartridge(Cartridge::new(&[0x6A, 0x15]));
        cpu.execute_cycle();
        assert_eq!(cpu.v[10], 0x15, "V10 is set");
        assert_eq!(cpu.pc, 0x202, "the program counter is advanced two bytes");
    }

    #[test]
    fn opcode_axxx() {
        let mut cpu = Cpu::new();
        cpu.load_cartridge(Cartridge::new(&[0xAF, 0xAF]));
        cpu.execute_cycle();

        assert_eq!(cpu.i, 0x0FAF, "the 'i' register is updated");
        assert_eq!(cpu.pc, 0x202, "the program counter is advanced two bytes");
    }
}
