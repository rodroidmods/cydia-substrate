pub fn is_arm_pc_relative(instruction: u32) -> bool {
    (instruction & 0x0c000000) == 0x04000000
        && (instruction & 0xf0000000) != 0xf0000000
        && (instruction & 0x000f0000) == 0x000f0000
}

pub fn is_thumb_32bit(instruction: u16) -> bool {
    (instruction & 0xe000) == 0xe000 && (instruction & 0x1800) != 0x0000
}

pub fn is_thumb_pc_relative_cbz(instruction: u16) -> bool {
    (instruction & 0xf500) == 0xb100
}

pub fn is_thumb_pc_relative_b(instruction: u16) -> bool {
    (instruction & 0xf000) == 0xd000 && (instruction & 0x0e00) != 0x0e00
}

pub fn is_thumb2_pc_relative_b(instructions: &[u16]) -> bool {
    if instructions.len() < 2 {
        return false;
    }
    (instructions[0] & 0xf800) == 0xf000
        && ((instructions[1] & 0xd000) == 0x9000 || (instructions[1] & 0xd000) == 0x8000)
        && (instructions[0] & 0x0380) != 0x0380
}

pub fn is_thumb_pc_relative_bl(instructions: &[u16]) -> bool {
    if instructions.len() < 2 {
        return false;
    }
    (instructions[0] & 0xf800) == 0xf000
        && ((instructions[1] & 0xd000) == 0xd000 || (instructions[1] & 0xd001) == 0xc000)
}

pub fn is_thumb_pc_relative_ldr(instruction: u16) -> bool {
    (instruction & 0xf800) == 0x4800
}

pub fn is_thumb_pc_relative_add(instruction: u16) -> bool {
    (instruction & 0xff78) == 0x4478
}

pub fn is_thumb_pc_relative_ldrw(instruction: u16) -> bool {
    (instruction & 0xff7f) == 0xf85f
}

pub fn get_thumb_instruction_width(start: *const u8) -> usize {
    unsafe {
        let thumb = start as *const u16;
        if is_thumb_32bit(*thumb) {
            4
        } else {
            2
        }
    }
}

pub fn get_arm_instruction_width(_start: *const u8) -> usize {
    4
}

pub fn get_instruction_width(start: *const u8) -> usize {
    if (start as usize & 0x1) == 0 {
        get_arm_instruction_width(start)
    } else {
        get_thumb_instruction_width((start as usize & !0x1) as *const u8)
    }
}
