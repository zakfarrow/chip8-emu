use rand::Rng;
use std::fs::File;
use std::io::Read;

pub const MEMORY_SIZE: usize = 4096;
pub const NUM_REGISTERS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const NUM_KEYS: usize = 16;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const SCALE_FACTOR: usize = 10;

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; NUM_REGISTERS],
    index: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; NUM_KEYS],
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    rng: rand::rngs::ThreadRng,
}

impl Chip8 {
    // ... [Previous implementation remains the same up to store_bcd]
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE],
            registers: [0; NUM_REGISTERS],
            index: 0,
            pc: 0x200,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; NUM_KEYS],
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            rng: rand::thread_rng(),
        };

        let fontset: [u8; 80] = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for i in 0..80 {
            chip8.memory[i] = fontset[i];
        }

        chip8
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        for (i, &byte) in buffer.iter().enumerate() {
            if 0x200 + i < MEMORY_SIZE {
                self.memory[0x200 + i] = byte;
            }
        }
        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 
                   | (self.memory[(self.pc + 1) as usize] as u16);

        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode & 0x00FF {
                0xE0 => self.clear_display(),
                0xEE => self.return_from_subroutine(),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            0x1 => self.jump(opcode & 0x0FFF),
            0x2 => self.call_subroutine(opcode & 0x0FFF),
            0x3 => self.skip_if_equal(
                ((opcode & 0x0F00) >> 8) as usize,
                (opcode & 0x00FF) as u8
            ),
            0x4 => self.skip_if_not_equal(
                ((opcode & 0x0F00) >> 8) as usize,
                (opcode & 0x00FF) as u8
            ),
            0x5 => self.skip_if_registers_equal(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize
            ),
            0x6 => self.set_register(
                ((opcode & 0x0F00) >> 8) as usize,
                (opcode & 0x00FF) as u8
            ),
            0x7 => self.add_to_register(
                ((opcode & 0x0F00) >> 8) as usize,
                (opcode & 0x00FF) as u8
            ),
            0x8 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    0x0 => self.set_register_to_register(x, y),
                    0x1 => self.or_registers(x, y),
                    0x2 => self.and_registers(x, y),
                    0x3 => self.xor_registers(x, y),
                    0x4 => self.add_registers(x, y),
                    0x5 => self.sub_registers(x, y),
                    0x6 => self.shift_right(x),
                    0x7 => self.sub_registers_reverse(x, y),
                    0xE => self.shift_left(x),
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            },
            0x9 => self.skip_if_registers_not_equal(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize
            ),
            0xA => self.set_index(opcode & 0x0FFF),
            0xB => self.jump_with_offset(opcode & 0x0FFF),
            0xC => self.random(
                ((opcode & 0x0F00) >> 8) as usize,
                (opcode & 0x00FF) as u8
            ),
            0xD => self.draw_sprite(
                ((opcode & 0x0F00) >> 8) as usize,
                ((opcode & 0x00F0) >> 4) as usize,
                (opcode & 0x000F) as u8
            ),
            0xE => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x9E => self.skip_if_key_pressed(x),
                    0xA1 => self.skip_if_key_not_pressed(x),
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            },
            0xF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x07 => self.set_register_to_delay_timer(x),
                    0x0A => self.wait_for_key(x),
                    0x15 => self.set_delay_timer(x),
                    0x18 => self.set_sound_timer(x),
                    0x1E => self.add_to_index(x),
                    0x29 => self.set_index_to_sprite(x),
                    0x33 => self.store_bcd(x),
                    0x55 => self.store_registers(x),
                    0x65 => self.load_registers(x),
                    _ => println!("Unknown opcode: {:X}", opcode),
                }
            },
            _ => println!("Unknown opcode: {:X}", opcode),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // Original methods remain the same...

    // New opcode implementations
    fn skip_if_registers_equal(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn set_register_to_register(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
        self.pc += 2;
    }

    fn or_registers(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
        self.pc += 2;
    }

    fn and_registers(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
        self.pc += 2;
    }

    fn xor_registers(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
        self.pc += 2;
    }

    fn add_registers(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[0xF] = if overflow { 1 } else { 0 };
        self.registers[x] = result;
        self.pc += 2;
    }

    fn sub_registers(&mut self, x: usize, y: usize) {
        self.registers[0xF] = if self.registers[x] > self.registers[y] { 1 } else { 0 };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        self.pc += 2;
    }

    fn shift_right(&mut self, x: usize) {
        self.registers[0xF] = self.registers[x] & 1;
        self.registers[x] >>= 1;
        self.pc += 2;
    }

    fn sub_registers_reverse(&mut self, x: usize, y: usize) {
        self.registers[0xF] = if self.registers[y] > self.registers[x] { 1 } else { 0 };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
        self.pc += 2;
    }

    fn shift_left(&mut self, x: usize) {
        self.registers[0xF] = (self.registers[x] & 0x80) >> 7;
        self.registers[x] <<= 1;
        self.pc += 2;
    }

    fn skip_if_registers_not_equal(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn jump_with_offset(&mut self, addr: u16) {
        self.pc = addr + self.registers[0] as u16;
    }

    fn random(&mut self, x: usize, nn: u8) {
        self.registers[x] = self.rng.gen::<u8>() & nn;
        self.pc += 2;
    }

    fn skip_if_key_pressed(&mut self, x: usize) {
        if self.keys[self.registers[x] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn skip_if_key_not_pressed(&mut self, x: usize) {
        if !self.keys[self.registers[x] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn set_register_to_delay_timer(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
        self.pc += 2;
    }

    fn wait_for_key(&mut self, x: usize) {
        let mut key_pressed = false;
        for i in 0..NUM_KEYS {
            if self.keys[i] {
                self.registers[x] = i as u8;
                key_pressed = true;
                break;
            }
        }
        if !key_pressed {
            return; // Don't increment PC, try again next cycle
        }
        self.pc += 2;
    }

    fn set_delay_timer(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
        self.pc += 2;
    }

    fn set_sound_timer(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
        self.pc += 2;
    }

    fn add_to_index(&mut self, x: usize) {
        self.index = self.index.wrapping_add(self.registers[x] as u16);
        self.pc += 2;
    }

    fn set_index_to_sprite(&mut self, x: usize) {
        self.index = (self.registers[x] as u16) * 5;
        self.pc += 2;
    }

    fn store_bcd(&mut self, x: usize) {
        let value = self.registers[x];
        self.memory[self.index as usize] = value / 100;
        self.memory[self.index as usize + 1] = (value / 10) % 10;
        self.memory[self.index as usize + 2] = value % 10;
        self.pc += 2;
    }
    fn store_registers(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.index as usize + i] = self.registers[i];
        }
        self.pc += 2;
    }

    fn load_registers(&mut self, x: usize) {
        for i in 0..=x {
            self.registers[i] = self.memory[self.index as usize + i];
        }
        self.pc += 2;
    }

    fn clear_display(&mut self) {
        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        self.pc += 2;
    }

    fn return_from_subroutine(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        self.pc += 2;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn skip_if_equal(&mut self, reg: usize, val: u8) {
        if self.registers[reg] == val {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn skip_if_not_equal(&mut self, reg: usize, val: u8) {
        if self.registers[reg] != val {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn set_register(&mut self, reg: usize, val: u8) {
        self.registers[reg] = val;
        self.pc += 2;
    }

    fn add_to_register(&mut self, reg: usize, val: u8) {
        self.registers[reg] = self.registers[reg].wrapping_add(val);
        self.pc += 2;
    }

    fn set_index(&mut self, val: u16) {
        self.index = val;
        self.pc += 2;
    }

    fn draw_sprite(&mut self, x_reg: usize, y_reg: usize, height: u8) {
        let x = self.registers[x_reg] as usize % DISPLAY_WIDTH;
        let y = self.registers[y_reg] as usize % DISPLAY_HEIGHT;
        self.registers[0xF] = 0;

        for row in 0..height {
            let y_pos = (y + row as usize) % DISPLAY_HEIGHT;
            let pixel = self.memory[(self.index + row as u16) as usize];

            for col in 0..8 {
                let x_pos = (x + col) % DISPLAY_WIDTH;
                if (pixel & (0x80 >> col)) != 0 {
                    if self.display[y_pos][x_pos] {
                        self.registers[0xF] = 1;
                    }
                    self.display[y_pos][x_pos] ^= true;
                }
            }
        }
        self.pc += 2;
    }

    pub fn get_display_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT * SCALE_FACTOR * SCALE_FACTOR];
        
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let color = if self.display[y][x] { 0xFFFFFF } else { 0 };
                
                for scale_y in 0..SCALE_FACTOR {
                    for scale_x in 0..SCALE_FACTOR {
                        let idx = (y * SCALE_FACTOR + scale_y) * (DISPLAY_WIDTH * SCALE_FACTOR) 
                               + (x * SCALE_FACTOR + scale_x);
                        buffer[idx] = color;
                    }
                }
            }
        }
        
        buffer
    }

    pub fn key_press(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }
}