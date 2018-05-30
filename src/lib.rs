extern crate rand;

mod opcodes;

use opcodes::Opcode;
use rand::distributions::{Uniform, Distribution};
use std::io::{ Read, Result };

pub struct Chip8 {
    instruction: u16,
    opcode: Opcode,
    memory: [u8; 4096],
    regs: [u8; 16],
    i: usize,
    pc: usize,
    pub gfx: [bool; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    sp: usize,
    keys: [bool; 16],
    pub draw_flag: bool,
    uniform:  Uniform<u16>,
    rng: rand::ThreadRng,
}

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80  //F
];

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c8 = Chip8 {
            instruction: 0,
            opcode: Opcode::Cls,
            memory: [0; 4096],
            regs: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [false; 16],
            draw_flag: false,
            uniform: Uniform::from(0..256),
            rng: rand::thread_rng(),
        };
        c8.memory[..80].copy_from_slice(&FONT_SET);
        c8
    }

    pub fn load(&mut self, input: &mut impl Read) -> Result<()> {
        input.read(&mut self.memory[0x200..])?;
        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.fetch();
        self.decode();
        self.execute();
    }

    #[inline]
    fn fetch(&mut self) {
        self.instruction = u16::from(self.memory[self.pc]) << 8
            | u16::from(self.memory[self.pc + 1]);
        self.pc += 2;
    }

    #[inline]
    fn decode(&mut self) {
        self.opcode = Opcode::from_u16(self.instruction).unwrap();
    }

    fn execute(&mut self) {
        use opcodes::Opcode::*;

        match self.opcode {
            CallProgram(addr)       => self.pc = addr as usize,
            Cls                     => self.gfx = [false; 2048],
            Ret                     => self.ret(),
            Jmp(addr)               => self.pc = addr as usize,
            Call(addr)              => self.call(addr),
            SkipEq(reg, imm)        => if self.regs[reg as usize] == imm { self.pc += 2; },
            SkipNeq(reg, imm)       => if self.regs[reg as usize] != imm { self.pc += 2; },
            SkipEqR(reg_x, reg_y)   => if self.regs[reg_x as usize] == self.regs[reg_y as usize] { self.pc += 2; },
            SkipNeqR(reg_x, reg_y)  => if self.regs[reg_x as usize] != self.regs[reg_y as usize] { self.pc += 2; },
            Mv(reg, imm)            => self.regs[reg as usize] = imm,
            Add(reg, imm)           => self.regs[reg as usize] =  self.regs[reg as usize].wrapping_add(imm),
            MvR(reg_x, reg_y)       => self.regs[reg_x as usize] = self.regs[reg_y as usize],
            MvOr(reg_x, reg_y)      => self.regs[reg_x as usize] |= self.regs[reg_y as usize],
            MvAnd(reg_x, reg_y)     => self.regs[reg_x as usize] &= self.regs[reg_y as usize],
            MvXor(reg_x, reg_y)     => self.regs[reg_x as usize] ^= self.regs[reg_y as usize],
            AddC(reg_x, reg_y)      => self.add_with_carry(reg_x as usize, reg_y as usize),
            SubC(reg_x, reg_y)      => self.sub_with_carry(reg_x as usize, reg_y as usize),
            Shr(reg_x, reg_y)       => self.shr(reg_x as usize, reg_y as usize),
            SubIC(reg_x, reg_y)     => self.sub_ic(reg_x as usize, reg_y as usize),
            Shl(reg_x, reg_y)       => self.shl(reg_x as usize, reg_y as usize),
            SetI(addr)              => self.i = addr as usize,
            JmpV0(addr)             => self.pc = (addr as usize).wrapping_add(self.regs[0] as usize),
            SetRnd(reg, imm)        => self.regs[reg as usize] = self.uniform.sample(&mut self.rng) as u8 & imm,
            Draw(reg_x, reg_y, height) => self.draw(reg_x as usize, reg_y as usize, height as usize),
            SkipKeyPrssd(reg)       => if self.keys[self.regs[reg as usize] as usize] { self.pc += 2 },
            SkipKeyNPrssd(reg)      => if !self.keys[self.regs[reg as usize] as usize] { self.pc += 2 },
            DelayTimer(reg)         => self.regs[reg as usize] = self.delay_timer,
            GetKey(reg)             => self.get_key(reg as usize),
            SetDelay(reg)           => self.delay_timer = self.regs[reg as usize],
            SetSound(reg)           => self.sound_timer = self.regs[reg as usize],
            AddVToI(reg)            => self.i += self.regs[reg as usize] as usize,
            SetIChr(reg)            => self.i = usize::from(self.regs[reg as usize]) * 5,
            BCD(reg)                => self.bcd(reg as usize),
            RegDump(reg)            => self.reg_dump(reg as usize),
            RegLoad(reg)            => self.reg_load(reg as usize),
        }
    }

    fn ret(&mut self) {
        if self.sp == 0 {
            panic!("Returning from base routine at address: {}", self.pc);
        }

        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn call(&mut self, addr: u16) {
        if self.sp == 15 {
            panic!("Stack overflow at address: {}", self.pc);
        }

        self.stack[self.sp] = self.pc;
        self.pc = addr as usize;
        self.sp += 1;
    }

    fn add_with_carry(&mut self, reg_x: usize, reg_y: usize) {
        let (res, carry) = self.regs[reg_x].overflowing_add(self.regs[reg_y]);
        self.regs[reg_x] = res;
        self.regs[15] = carry as u8;
    }

    fn sub_with_carry(&mut self, reg_x: usize, reg_y: usize) {
        let (res, borrow) = self.regs[reg_x].overflowing_sub(self.regs[reg_y]);
        self.regs[reg_x] = res;
        self.regs[15] = !borrow as u8;
    }

    fn shr(&mut self, reg_x: usize, reg_y: usize) {
        let reg_y_val = self.regs[reg_y];
        self.regs[reg_x] = reg_y_val >> 1;
        self.regs[15] = reg_y_val & 1;
    }

    fn sub_ic(&mut self, reg_x: usize, reg_y: usize) {
        let (res, borrow) = self.regs[reg_y].overflowing_sub(self.regs[reg_x]);
        self.regs[reg_x] = res;
        self.regs[15] = !borrow as u8;
    }

    fn shl(&mut self, reg_x: usize, reg_y: usize) {
        let reg_y_val = self.regs[reg_y];
        let bit = reg_y_val >> 7;
        self.regs[reg_x] = reg_y_val << 1;
        self.regs[reg_y] = reg_y_val << 1;
        self.regs[15] = bit;
    }

    fn draw(&mut self, reg_x: usize, reg_y: usize, height: usize) {
        let x = self.regs[reg_x] as usize;
        let y = self.regs[reg_y] as usize;
        self.regs[15] = 0;
        let mut set_to_unset = false;
        let mut unset_to_set = false;
        for y_line in 0..height {
            let pixel = self.memory[self.i + y_line];
            for x_line in 0..8 {
                if pixel & (0x80 >> x_line) != 0 {
                    let pixel_index = x + x_line + (y + y_line) * 64;
                    if pixel_index >= self.gfx.len() {
                        continue;
                    }
                    if self.gfx[pixel_index] {
                        set_to_unset = true;
                    } else {
                        unset_to_set  = true;
                    }
                    self.gfx[pixel_index] = !self.gfx[pixel_index];
                }
            }
        }
        self.regs[15] = set_to_unset as u8;
        self.draw_flag = unset_to_set;
    }

    fn get_key(&mut self, reg: usize) {
        let mut key_pressed = false;
        for (key, pressed) in self.keys.iter().enumerate() {
            if *pressed {
                self.regs[reg] = key as u8;
                key_pressed = true;
            }
        }

        // Try again
        if !key_pressed {
            self.pc -= 2;
        }
    }

    fn bcd(&mut self, reg: usize) {
        let mut val = self.regs[reg as usize];
        self.memory[self.i] = val / 100;
        self.memory[self.i + 2] = val % 10;
        val /= 10;
        self.memory[self.i + 1] = val % 10;
    }

    fn reg_dump(&mut self, reg: usize) {
        for i in 0..=reg {
            self.memory[self.i] = self.regs[i];
            self.i += 1;
        }
    }

    fn reg_load(&mut self, reg: usize) {
        for i in 0 ..= reg {
            self.regs[i] = self.memory[self.i];
            self.i += 1;
        }
    }

    pub fn press_key(&mut self, key: usize) {
        self.keys[key] = true;
    }

    pub fn release_key(&mut self, key: usize) {
        self.keys[key] = false;
    }

    pub fn decrease_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn render(&mut self) {
        self.draw_flag = true;
    }
}