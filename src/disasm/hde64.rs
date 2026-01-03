const C_MODRM: u8 = 0x01;
const C_IMM8: u8 = 0x02;
const C_IMM16: u8 = 0x04;
const C_IMM_P66: u8 = 0x10;
const C_REL8: u8 = 0x20;
const C_REL32: u8 = 0x40;
const C_GROUP: u8 = 0x80;
const C_ERROR: u8 = 0xff;

const PRE_NONE: u8 = 0x01;
const PRE_F2: u8 = 0x02;
const PRE_F3: u8 = 0x04;
const PRE_66: u8 = 0x08;
const PRE_67: u8 = 0x10;
const PRE_LOCK: u8 = 0x20;
const PRE_SEG: u8 = 0x40;

const DELTA_OPCODES: usize = 0x4a;
const DELTA_FPU_REG: usize = 0xfd;
const DELTA_FPU_MODRM: usize = 0x104;
const DELTA_PREFIXES: usize = 0x13c;
const DELTA_OP_LOCK_OK: usize = 0x1ae;
const DELTA_OP2_LOCK_OK: usize = 0x1c6;
const DELTA_OP_ONLY_MEM: usize = 0x1d8;
const DELTA_OP2_ONLY_MEM: usize = 0x1e7;

const HDE64_TABLE: &[u8] = &[
  0xa5,0xaa,0xa5,0xb8,0xa5,0xaa,0xa5,0xaa,0xa5,0xb8,0xa5,0xb8,0xa5,0xb8,0xa5,
  0xb8,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xc0,0xac,0xc0,0xcc,0xc0,0xa1,0xa1,
  0xa1,0xa1,0xb1,0xa5,0xa5,0xa6,0xc0,0xc0,0xd7,0xda,0xe0,0xc0,0xe4,0xc0,0xea,
  0xea,0xe0,0xe0,0x98,0xc8,0xee,0xf1,0xa5,0xd3,0xa5,0xa5,0xa1,0xea,0x9e,0xc0,
  0xc0,0xc2,0xc0,0xe6,0x03,0x7f,0x11,0x7f,0x01,0x7f,0x01,0x3f,0x01,0x01,0xab,
  0x8b,0x90,0x64,0x5b,0x5b,0x5b,0x5b,0x5b,0x92,0x5b,0x5b,0x76,0x90,0x92,0x92,
  0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x6a,0x73,0x90,
  0x5b,0x52,0x52,0x52,0x52,0x5b,0x5b,0x5b,0x5b,0x77,0x7c,0x77,0x85,0x5b,0x5b,
  0x70,0x5b,0x7a,0xaf,0x76,0x76,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,0x5b,
  0x5b,0x5b,0x86,0x01,0x03,0x01,0x04,0x03,0xd5,0x03,0xd5,0x03,0xcc,0x01,0xbc,
  0x03,0xf0,0x03,0x03,0x04,0x00,0x50,0x50,0x50,0x50,0xff,0x20,0x20,0x20,0x20,
  0x01,0x01,0x01,0x01,0xc4,0x02,0x10,0xff,0xff,0xff,0x01,0x00,0x03,0x11,0xff,
  0x03,0xc4,0xc6,0xc8,0x02,0x10,0x00,0xff,0xcc,0x01,0x01,0x01,0x00,0x00,0x00,
  0x00,0x01,0x01,0x03,0x01,0xff,0xff,0xc0,0xc2,0x10,0x11,0x02,0x03,0x01,0x01,
  0x01,0xff,0xff,0xff,0x00,0x00,0x00,0xff,0x00,0x00,0xff,0xff,0xff,0xff,0x10,
  0x10,0x10,0x10,0x02,0x10,0x00,0x00,0xc6,0xc8,0x02,0x02,0x02,0x02,0x06,0x00,
  0x04,0x00,0x02,0xff,0x00,0xc0,0xc2,0x01,0x01,0x03,0x03,0x03,0xca,0x40,0x00,
  0x0a,0x00,0x04,0x00,0x00,0x00,0x00,0x7f,0x00,0x33,0x01,0x00,0x00,0x00,0x00,
  0x00,0x00,0xff,0xbf,0xff,0xff,0x00,0x00,0x00,0x00,0x07,0x00,0x00,0xff,0x00,
  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xff,0xff,
  0x00,0x00,0x00,0xbf,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x7f,0x00,0x00,
  0xff,0x40,0x40,0x40,0x40,0x41,0x49,0x40,0x40,0x40,0x40,0x4c,0x42,0x40,0x40,
  0x40,0x40,0x40,0x40,0x40,0x40,0x4f,0x44,0x53,0x40,0x40,0x40,0x44,0x57,0x43,
  0x5c,0x40,0x60,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,
  0x40,0x40,0x64,0x66,0x6e,0x6b,0x40,0x40,0x6a,0x46,0x40,0x40,0x44,0x46,0x40,
  0x40,0x5b,0x44,0x40,0x40,0x00,0x00,0x00,0x00,0x06,0x06,0x06,0x06,0x01,0x06,
  0x06,0x02,0x06,0x06,0x00,0x06,0x00,0x0a,0x0a,0x00,0x00,0x00,0x02,0x07,0x07,
  0x06,0x02,0x0d,0x06,0x06,0x06,0x0e,0x05,0x05,0x02,0x02,0x00,0x00,0x04,0x04,
  0x04,0x04,0x05,0x06,0x06,0x06,0x00,0x00,0x00,0x0e,0x00,0x00,0x08,0x00,0x10,
  0x00,0x18,0x00,0x20,0x00,0x28,0x00,0x30,0x00,0x80,0x01,0x82,0x01,0x86,0x00,
  0xf6,0xcf,0xfe,0x3f,0xab,0x00,0xb0,0x00,0xb1,0x00,0xb3,0x00,0xba,0xf8,0xbb,
  0x00,0xc0,0x00,0xc1,0x00,0xc7,0xbf,0x62,0xff,0x00,0x8d,0xff,0x00,0xc4,0xff,
  0x00,0xc5,0xff,0x00,0xff,0xff,0xeb,0x01,0xff,0x0e,0x12,0x08,0x00,0x13,0x09,
  0x00,0x16,0x08,0x00,0x17,0x09,0x00,0x2b,0x09,0x00,0xae,0xff,0x07,0xb2,0xff,
  0x00,0xb4,0xff,0x00,0xb5,0xff,0x00,0xc3,0x01,0x00,0xc7,0xff,0xbf,0xe7,0x08,
  0x00,0xf0,0x02,0x00
];

pub const F_MODRM: u32 = 0x00000001;
pub const F_SIB: u32 = 0x00000002;
pub const F_IMM8: u32 = 0x00000004;
pub const F_IMM16: u32 = 0x00000008;
pub const F_IMM32: u32 = 0x00000010;
pub const F_IMM64: u32 = 0x00000020;
pub const F_DISP8: u32 = 0x00000040;
pub const F_DISP16: u32 = 0x00000080;
pub const F_DISP32: u32 = 0x00000100;
pub const F_RELATIVE: u32 = 0x00000200;
pub const F_ERROR: u32 = 0x00001000;
pub const F_ERROR_OPCODE: u32 = 0x00002000;
pub const F_ERROR_LENGTH: u32 = 0x00004000;
pub const F_ERROR_LOCK: u32 = 0x00008000;
pub const F_ERROR_OPERAND: u32 = 0x00010000;
pub const F_PREFIX_REPNZ: u32 = 0x01000000;
pub const F_PREFIX_REPX: u32 = 0x02000000;
pub const F_PREFIX_REP: u32 = 0x03000000;
pub const F_PREFIX_66: u32 = 0x04000000;
pub const F_PREFIX_67: u32 = 0x08000000;
pub const F_PREFIX_LOCK: u32 = 0x10000000;
pub const F_PREFIX_SEG: u32 = 0x20000000;
pub const F_PREFIX_REX: u32 = 0x40000000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Hde64s {
    pub len: u8,
    pub p_rep: u8,
    pub p_lock: u8,
    pub p_seg: u8,
    pub p_66: u8,
    pub p_67: u8,
    pub rex: u8,
    pub rex_w: u8,
    pub rex_r: u8,
    pub rex_x: u8,
    pub rex_b: u8,
    pub opcode: u8,
    pub opcode2: u8,
    pub modrm: u8,
    pub modrm_mod: u8,
    pub modrm_reg: u8,
    pub modrm_rm: u8,
    pub sib: u8,
    pub sib_scale: u8,
    pub sib_index: u8,
    pub sib_base: u8,
    pub imm: ImmUnion,
    pub disp: DispUnion,
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ImmUnion {
    pub imm8: u8,
    pub imm16: u16,
    pub imm32: u32,
    pub imm64: u64,
}

impl std::fmt::Debug for ImmUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImmUnion")
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DispUnion {
    pub disp8: u8,
    pub disp16: u16,
    pub disp32: u32,
}

impl std::fmt::Debug for DispUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DispUnion")
    }
}

pub unsafe fn hde64_disasm(code: *const u8, hs: *mut Hde64s) -> u32 { unsafe {
    let mut p = code;
    std::ptr::write_bytes(hs, 0, 1);

    let mut pref: u8 = 0;
    let mut cflags: u8;
    let opcode: u8;
    let mut c: u8;
    let mut x: u8;
    let mut op64: u8 = 0;
    let mut m_mod: u8;
    let m_reg: u8;
    let m_rm: u8;
    let mut disp_size: u8 = 0;
    let mut _ht: usize = 0;

    for _ in 0..16 {
        c = *p;
        p = p.add(1);
        match c {
            0xf3 => {
                (*hs).p_rep = c;
                pref |= PRE_F3;
            }
            0xf2 => {
                (*hs).p_rep = c;
                pref |= PRE_F2;
            }
            0xf0 => {
                (*hs).p_lock = c;
                pref |= PRE_LOCK;
            }
            0x26 | 0x2e | 0x36 | 0x3e | 0x64 | 0x65 => {
                (*hs).p_seg = c;
                pref |= PRE_SEG;
            }
            0x66 => {
                (*hs).p_66 = c;
                pref |= PRE_66;
            }
            0x67 => {
                (*hs).p_67 = c;
                pref |= PRE_67;
            }
            _ => break,
        }
    }

    (*hs).flags = (pref as u32) << 23;

    if pref == 0 {
        pref |= PRE_NONE;
    }

    c = *p.sub(1);
    if (c & 0xf0) == 0x40 {
        (*hs).flags |= F_PREFIX_REX;
        (*hs).rex_w = (c & 0xf) >> 3;
        if (*hs).rex_w != 0 && (*p & 0xf8) == 0xb8 {
            op64 += 1;
        }
        (*hs).rex_r = (c & 7) >> 2;
        (*hs).rex_x = (c & 3) >> 1;
        (*hs).rex_b = c & 1;
        c = *p;
        p = p.add(1);
        if (c & 0xf0) == 0x40 {
            opcode = c;
            (*hs).flags |= F_ERROR | F_ERROR_OPCODE;
            (*hs).opcode = opcode;
            (*hs).len = (p as usize - code as usize) as u8;
            return (*hs).len as u32;
        }
    }

    (*hs).opcode = c;
    if c == 0x0f {
        (*hs).opcode2 = *p;
        p = p.add(1);
        _ht = DELTA_OPCODES;
        c = (*hs).opcode2;
    } else if c >= 0xa0 && c <= 0xa3 {
        op64 += 1;
        if (pref & PRE_67) != 0 {
            pref |= PRE_66;
        } else {
            pref &= !PRE_66;
        }
    }

    opcode = c;
    cflags = HDE64_TABLE[HDE64_TABLE[(opcode / 4) as usize] as usize + (opcode % 4) as usize];

    if cflags == C_ERROR {
        (*hs).flags |= F_ERROR | F_ERROR_OPCODE;
        cflags = 0;
        if (opcode & 0xfd) == 0x24 {
            cflags += 1;
        }
    }

    x = 0;
    if (cflags & C_GROUP) != 0 {
        let t_offset = (cflags & 0x7f) as usize;
        let t = u16::from_le_bytes([
            HDE64_TABLE[t_offset],
            HDE64_TABLE[t_offset + 1]
        ]);
        cflags = t as u8;
        x = (t >> 8) as u8;
    }

    if (*hs).opcode2 != 0 {
        let ht_base = DELTA_PREFIXES;
        let ht_val = HDE64_TABLE[ht_base + HDE64_TABLE[ht_base + (opcode / 4) as usize] as usize + (opcode % 4) as usize];
        if (ht_val & pref) != 0 {
            (*hs).flags |= F_ERROR | F_ERROR_OPCODE;
        }
    }

    if (cflags & C_MODRM) != 0 {
        (*hs).flags |= F_MODRM;
        (*hs).modrm = *p;
        p = p.add(1);
        c = (*hs).modrm;
        (*hs).modrm_mod = c >> 6;
        (*hs).modrm_rm = c & 7;
        (*hs).modrm_reg = (c & 0x3f) >> 3;
        m_mod = (*hs).modrm_mod;
        m_rm = (*hs).modrm_rm;
        m_reg = (*hs).modrm_reg;

        if x != 0 && ((x << m_reg) & 0x80) != 0 {
            (*hs).flags |= F_ERROR | F_ERROR_OPCODE;
        }

        if (*hs).opcode2 == 0 && opcode >= 0xd9 && opcode <= 0xdf {
            let t = opcode - 0xd9;
            let ht_val = if m_mod == 3 {
                let ht_base = DELTA_FPU_MODRM + (t as usize) * 8;
                HDE64_TABLE[ht_base + m_reg as usize] << m_rm
            } else {
                let ht_base = DELTA_FPU_REG;
                HDE64_TABLE[ht_base + t as usize] << m_reg
            };
            if (ht_val & 0x80) != 0 {
                (*hs).flags |= F_ERROR | F_ERROR_OPCODE;
            }
        }

        if (pref & PRE_LOCK) != 0 {
            if m_mod == 3 {
                (*hs).flags |= F_ERROR | F_ERROR_LOCK;
            } else {
                let (ht_start, ht_end, op) = if (*hs).opcode2 != 0 {
                    (DELTA_OP2_LOCK_OK, DELTA_OP_ONLY_MEM, opcode)
                } else {
                    (DELTA_OP_LOCK_OK, DELTA_OP2_LOCK_OK, opcode & 0xfe)
                };

                let mut found = false;
                let mut ht_idx = ht_start;
                while ht_idx < ht_end {
                    if HDE64_TABLE[ht_idx] == op {
                        ht_idx += 1;
                        if ((HDE64_TABLE[ht_idx] << m_reg) & 0x80) == 0 {
                            found = true;
                            break;
                        }
                        break;
                    }
                    ht_idx += 2;
                }
                if !found {
                    (*hs).flags |= F_ERROR | F_ERROR_LOCK;
                }
            }
        }

        if (*hs).opcode2 != 0 {
            match opcode {
                0x20 | 0x22 => {
                    m_mod = 3;
                    if m_reg > 4 || m_reg == 1 {
                        (*hs).flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0x21 | 0x23 => {
                    m_mod = 3;
                    if m_reg == 4 || m_reg == 5 {
                        (*hs).flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                _ => {}
            }
        } else {
            match opcode {
                0x8c => {
                    if m_reg > 5 {
                        (*hs).flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                0x8e => {
                    if m_reg == 1 || m_reg > 5 {
                        (*hs).flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                }
                _ => {}
            }
        }

        if m_mod == 3 {
            let (ht_start, ht_end) = if (*hs).opcode2 != 0 {
                (DELTA_OP2_ONLY_MEM, HDE64_TABLE.len())
            } else {
                (DELTA_OP_ONLY_MEM, DELTA_OP2_ONLY_MEM)
            };

            let mut ht_idx = ht_start;
            while ht_idx < ht_end {
                if HDE64_TABLE[ht_idx] == opcode {
                    ht_idx += 1;
                    if (HDE64_TABLE[ht_idx] & pref) != 0 && ((HDE64_TABLE[ht_idx + 1] << m_reg) & 0x80) == 0 {
                        (*hs).flags |= F_ERROR | F_ERROR_OPERAND;
                    }
                    break;
                }
                ht_idx += 3;
            }
        }

        if (cflags & C_MODRM) != 0 {
            if m_mod != 3 {
                if m_rm == 4 {
                    (*hs).flags |= F_SIB;
                    (*hs).sib = *p;
                    p = p.add(1);
                    (*hs).sib_scale = (*hs).sib >> 6;
                    (*hs).sib_index = ((*hs).sib & 0x3f) >> 3;
                    (*hs).sib_base = (*hs).sib & 7;
                    if (*hs).sib_base == 5 && m_mod == 0 {
                        disp_size = 4;
                    }
                } else if m_rm == 5 && m_mod == 0 {
                    disp_size = 4;
                }

                if m_mod == 1 {
                    disp_size = 1;
                } else if m_mod == 2 {
                    disp_size = 4;
                }
            }
        }
    }

    if (cflags & C_IMM_P66) != 0 {
        if (cflags & C_REL32) != 0 {
            if (pref & PRE_66) != 0 {
                (*hs).flags |= F_IMM16 | F_RELATIVE;
                (*hs).imm.imm16 = u16::from_le_bytes([*p, *p.add(1)]);
                p = p.add(2);
                (*hs).len = (p as usize - code as usize) as u8;
                if (*hs).len > 0x0f {
                    (*hs).flags |= F_ERROR | F_ERROR_LENGTH;
                }
                return (*hs).len as u32;
            }
            (*hs).flags |= F_IMM32 | F_RELATIVE;
            (*hs).imm.imm32 = u32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]);
            p = p.add(4);
        } else {
            if op64 != 0 {
                (*hs).flags |= F_IMM64;
                (*hs).imm.imm64 = u64::from_le_bytes([
                    *p, *p.add(1), *p.add(2), *p.add(3),
                    *p.add(4), *p.add(5), *p.add(6), *p.add(7)
                ]);
                p = p.add(8);
            } else if (pref & PRE_66) != 0 {
                (*hs).flags |= F_IMM16;
                (*hs).imm.imm16 = u16::from_le_bytes([*p, *p.add(1)]);
                p = p.add(2);
            } else {
                (*hs).flags |= F_IMM32;
                (*hs).imm.imm32 = u32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]);
                p = p.add(4);
            }
        }
    }

    if (cflags & C_IMM16) != 0 {
        if ((*hs).flags & F_IMM32) != 0 {
            (*hs).flags |= F_IMM16;
            (*hs).imm.imm16 = u16::from_le_bytes([*p, *p.add(1)]);
            p = p.add(2);
        }
    }

    if (cflags & C_IMM8) != 0 {
        (*hs).flags |= F_IMM8;
        (*hs).imm.imm8 = *p;
        p = p.add(1);
    }

    if (cflags & C_REL32) != 0 {
        (*hs).flags |= F_IMM32 | F_RELATIVE;
        (*hs).imm.imm32 = u32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]);
        p = p.add(4);
    } else if (cflags & C_REL8) != 0 {
        (*hs).flags |= F_IMM8 | F_RELATIVE;
        (*hs).imm.imm8 = *p;
        p = p.add(1);
    }

    if disp_size != 0 {
        if disp_size == 1 {
            (*hs).flags |= F_DISP8;
            (*hs).disp.disp8 = *p;
            p = p.add(1);
        } else {
            (*hs).flags |= F_DISP32;
            (*hs).disp.disp32 = u32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]);
            p = p.add(4);
        }
    }

    (*hs).len = (p as usize - code as usize) as u8;
    if (*hs).len > 0x0f {
        (*hs).flags |= F_ERROR | F_ERROR_LENGTH;
    }

    (*hs).len as u32
}}
