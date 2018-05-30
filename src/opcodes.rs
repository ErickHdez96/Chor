// https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
#[derive(Debug, PartialEq)]
pub enum Opcode {
    CallProgram(u16),   // Call RCA 1802 program at address u16
    Cls,                // Clear the screen
    Ret,                // Return from subroutine
    Jmp(u16),           // Jump to address u16
    Call(u16),          // Call subroutine at u16
    SkipEq(u8, u8),     // Skip next instruction if reg(u8) == u8
    SkipNeq(u8, u8),    // Skip next instruction if reg(u8) != u8
    SkipEqR(u8, u8),    // Skip next instruction if reg(u8) == reg(u8)
    SkipNeqR(u8, u8),   // Skip next instruction if reg(u8) != reg(u8)
    Mv(u8, u8),         // reg(u8) = u8
    Add(u8, u8),        // reg(u8) += u8
    MvR(u8, u8),        // reg(u8) = reg(u8)
    MvOr(u8, u8),       // reg(u8) |= reg(u8)
    MvAnd(u8, u8),      // reg(u8) &= reg(u8)
    MvXor(u8, u8),      // reg(u8) ^= reg(u8)
    AddC(u8, u8),       // reg(u8) += reg(u8) and VF = carry
    SubC(u8, u8),       // reg(u8) -= reg(u8) and VF = !borrow
    Shr(u8, u8),        // reg(u8) = reg(u8) >> 1 and VF = bit lost
    SubIC(u8, u8),      // reg(u8) = reg(u82) - reg(u81) and VF = !borrow
    Shl(u8, u8),        // reg(u81) = reg(u82) = reg(u82) << 1 and VF = bit lost
    SetI(u16),          // I = u16
    JmpV0(u16),         // Jump to address u16 + V0
    SetRnd(u8, u8),     // reg(u8) = rand(0, 256) & u8
    // Draw sprite at coordinate (u8, u8) with width 8 and height u8
    // Each row of 8 pixels is bit-coded starting at location I
    // VF = 1 if a pixel is set from set to unset, 0 otherwise
    Draw(u8, u8, u8),
    SkipKeyPrssd(u8),   // Skip next instruction if key in reg(u8) is pressed
    SkipKeyNPrssd(u8),  // Skip next instruction if key in reg(u8) is not pressed
    DelayTimer(u8),     // reg(u8) = delay_timer
    GetKey(u8),         // reg(u8) = GetKey() Blocking operation
    SetDelay(u8),       // delay_timer = reg(u8)
    SetSound(u8),       // sound_timer = reg(u8)
    AddVToI(u8),        // I += reg(u8)
    SetIChr(u8),        // I = location of the sprite for the character in reg(u8)
    // Store the BCD of reg(u8)
    // Hundreds at I
    // Tens at I + 1
    // Ones at I + 2
    BCD(u8),
    RegDump(u8),        // Store [reg(0),reg(u8)] to mem[I] increasing I
    RegLoad(u8),        // Fill [reg(0),reg(u8)] from mem[I] increasing I
}

impl Opcode {
    pub fn from_u16(inst: u16) -> Option<Opcode> {
        use self::Opcode::*;

        // Extract value form position &ing with mask
        macro_rules! extract {
            ( $pos:expr, $mask:expr ) => {{
                (inst >> ($pos << 2)) & $mask
            }}
        }

        // Register from position 2 and Imm from position 0
        macro_rules! xnn {
            ( $opcode:ident ) => {{
                $opcode(extract!(2, 0xF) as u8, extract!(0, 0xFF) as u8)
            }}
        }

        // Register form position 2 and 1
        macro_rules! xy0 {
            ( $opcode:ident) => {{
                $opcode(extract!(2, 0xF) as u8, extract!(1, 0xF) as u8)
            }}
        }

        // Imm from position 0
        macro_rules! nnn {
            ( $opcode:ident ) => {{
                $opcode(extract!(0, 0xFFF) as u16)
            }}
        }

        // Register form position 2
        macro_rules! x00 {
            ( $opcode:ident ) => {{
                $opcode(extract!(2, 0xF) as u8)
            }}
        }

        Some(match (inst >> 12) & 0xF {
            0x0 => match inst & 0x0FFF {
                0x00E0 => Cls,
                0x00EE => Ret,
                addr => CallProgram(addr),
            },
            0x1 => Jmp(inst & 0x0FFF),
            0x2 => Call(inst & 0x0FFF),
            0x3 => xnn!(SkipEq),
            0x4 => xnn!(SkipNeq),
            0x5 => xy0!(SkipEqR),
            0x6 => xnn!(Mv),
            0x7 => xnn!(Add),
            0x8 => match inst & 0xF {
                0x0 => xy0!(MvR),
                0x1 => xy0!(MvOr),
                0x2 => xy0!(MvAnd),
                0x3 => xy0!(MvXor),
                0x4 => xy0!(AddC),
                0x5 => xy0!(SubC),
                0x6 => xy0!(Shr),
                0x7 => xy0!(SubIC),
                0xE => xy0!(Shl),
                _ => return None,
            },
            0x9 => xy0!(SkipNeqR),
            0xA => nnn!(SetI),
            0xB => nnn!(JmpV0),
            0xC => xnn!(SetRnd),
            0xD => Draw(extract!(2, 0xF) as u8, extract!(1, 0xF) as u8, extract!(0, 0xF) as u8),
            0xE => match inst & 0x00FF {
                0x9E => x00!(SkipKeyPrssd),
                0xA1 => x00!(SkipKeyNPrssd),
                _ => return None,
            },
            0xF => match inst & 0x00FF {
                0x07 => x00!(DelayTimer),
                0x0A => x00!(GetKey),
                0x15 => x00!(SetDelay),
                0x18 => x00!(SetSound),
                0x1E => x00!(AddVToI),
                0x29 => x00!(SetIChr),
                0x33 => x00!(BCD),
                0x55 => x00!(RegDump),
                0x65 => x00!(RegLoad),
                _ => return None,
            },
            _ => return None,
        })
    }
}
