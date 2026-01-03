use crate::disasm::hde64::{hde64_disasm, Hde64s};
use crate::error::{Result, SubstrateError};
use crate::hook::memory::{allocate_trampoline, make_executable, ProtectedMemory};
use std::ptr;

#[cfg(target_pointer_width = "64")]
const IA32: bool = false;
#[cfg(target_pointer_width = "32")]
const IA32: bool = true;

fn is_32bit_offset(target: usize, source: usize) -> bool {
    let offset = target as isize - source as isize;
    offset as i32 as isize == offset
}

fn size_of_skip() -> usize {
    5
}

fn size_of_push_pointer(target: usize) -> usize {
    if (target >> 32) == 0 { 5 } else { 13 }
}

fn size_of_jump_blind(target: usize) -> usize {
    if IA32 {
        size_of_skip()
    } else {
        size_of_push_pointer(target) + 1
    }
}

fn size_of_jump(target: usize, source: usize) -> usize {
    if IA32 || is_32bit_offset(target, source + 5) {
        size_of_skip()
    } else {
        size_of_push_pointer(target) + 1
    }
}

fn size_of_pop(target: u8) -> usize {
    if (target >> 3) != 0 { 2 } else { 1 }
}

fn size_of_move64() -> usize {
    3
}

unsafe fn write_u8(current: &mut *mut u8, value: u8) {
    **current = value;
    *current = current.add(1);
}

unsafe fn write_u32(current: &mut *mut u8, value: u32) {
    ptr::write_unaligned(*current as *mut u32, value.to_le());
    *current = current.add(4);
}

unsafe fn write_i32(current: &mut *mut u8, value: i32) {
    write_u32(current, value as u32);
}

unsafe fn write_bytes(current: &mut *mut u8, data: *const u8, len: usize) {
    ptr::copy_nonoverlapping(data, *current, len);
    *current = current.add(len);
}

unsafe fn write_skip(current: &mut *mut u8, size: isize) {
    write_u8(current, 0xe9);
    write_i32(current, size as i32);
}

unsafe fn push_pointer(current: &mut *mut u8, target: usize) {
    write_u8(current, 0x68);
    write_u32(current, target as u32);

    let high = (target >> 32) as u32;
    if high != 0 {
        write_u8(current, 0xc7);
        write_u8(current, 0x44);
        write_u8(current, 0x24);
        write_u8(current, 0x04);
        write_u32(current, high);
    }
}

unsafe fn write_jump(current: &mut *mut u8, target: usize) {
    let source = *current as usize;

    if IA32 || is_32bit_offset(target, source + 5) {
        write_skip(current, (target as isize) - (source as isize) - 5);
    } else {
        push_pointer(current, target);
        write_u8(current, 0xc3);
    }
}

unsafe fn write_pop(current: &mut *mut u8, target: u8) {
    if (target >> 3) != 0 {
        write_u8(current, 0x40 | ((target & 0x08) >> 3));
    }
    write_u8(current, 0x58 | (target & 0x07));
}

unsafe fn write_move64(current: &mut *mut u8, source: u8, target: u8) {
    write_u8(current, 0x48 | ((target & 0x08) >> 3 << 2) | ((source & 0x08) >> 3));
    write_u8(current, 0x8b);
    write_u8(current, ((target & 0x07) << 3) | (source & 0x07));
}

pub unsafe fn hook_function_x86_64(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullPointer);
    }

    let source = symbol as usize;
    let target = replace as usize;

    let required = size_of_jump(target, source);

    let mut used = 0;
    while used < required {
        let mut decode = Hde64s {
            len: 0, p_rep: 0, p_lock: 0, p_seg: 0, p_66: 0, p_67: 0,
            rex: 0, rex_w: 0, rex_r: 0, rex_x: 0, rex_b: 0,
            opcode: 0, opcode2: 0, modrm: 0, modrm_mod: 0, modrm_reg: 0, modrm_rm: 0,
            sib: 0, sib_scale: 0, sib_index: 0, sib_base: 0,
            imm: crate::disasm::hde64::ImmUnion { imm8: 0 },
            disp: crate::disasm::hde64::DispUnion { disp8: 0 },
            flags: 0,
        };
        let width = hde64_disasm(symbol.add(used), &mut decode);
        if width == 0 {
            return Err(SubstrateError::DisassemblyFailed);
        }
        used += width as usize;
    }

    let blank = used - required;
    let mut backup = vec![0u8; used];
    ptr::copy_nonoverlapping(symbol, backup.as_mut_ptr(), used);

    if !result.is_null() {
        if backup[0] == 0xe9 {
            *result = (source + 5 + ptr::read_unaligned(backup.as_ptr().add(1) as *const u32) as usize) as *mut u8;
            return Ok(4);
        }

        if !IA32 && backup[0] == 0xff && backup[1] == 0x25 {
            *result = *((source + 6 + ptr::read_unaligned(backup.as_ptr().add(2) as *const u32) as usize) as *const *mut u8);
            return Ok(6);
        }

        let mut length = used + size_of_jump_blind(source + used);

        let mut offset = 0;
        while offset < used {
            let mut decode = std::mem::zeroed();
            hde64_disasm(backup.as_ptr().add(offset), &mut decode);
            let width = decode.len as usize;

            #[cfg(target_pointer_width = "64")]
            {
                if (decode.modrm & 0xc7) == 0x05 {
                    if decode.opcode == 0x8b {
                        let destiny = (symbol.add(offset).add(width) as isize + decode.disp.disp32 as i32 as isize) as *mut u8;
                        let reg = (decode.rex_r << 3) | decode.modrm_reg;
                        length -= decode.len as usize;
                        length += size_of_push_pointer(destiny as usize);
                        length += size_of_pop(reg);
                        length += size_of_move64();
                    }
                }
            }

            if backup[offset] == 0xe8 {
                let relative = ptr::read_unaligned(backup.as_ptr().add(offset + 1) as *const i32);
                let destiny = symbol.add(offset).add(decode.len as usize).offset(relative as isize);

                if relative == 0 {
                    length -= decode.len as usize;
                    length += size_of_push_pointer(destiny as usize);
                } else {
                    length += size_of_skip();
                    length += size_of_jump_blind(destiny as usize);
                }
            } else if backup[offset] == 0xeb {
                length -= decode.len as usize;
                length += size_of_jump_blind((symbol.add(offset).add(decode.len as usize) as isize + *backup.as_ptr().add(offset + 1) as i8 as isize) as usize);
            } else if backup[offset] == 0xe9 {
                length -= decode.len as usize;
                length += size_of_jump_blind((symbol.add(offset).add(decode.len as usize) as isize + ptr::read_unaligned(backup.as_ptr().add(offset + 1) as *const i32) as isize) as usize);
            } else if backup[offset] == 0xe3 || (backup[offset] & 0xf0) == 0x70 {
                length += decode.len as usize;
                length += size_of_jump_blind((symbol.add(offset).add(decode.len as usize) as isize + *backup.as_ptr().add(offset + 1) as i8 as isize) as usize);
            }

            offset += width;
        }

        let buffer = allocate_trampoline(length)?;
        let mut current = buffer;

        let mut offset = 0;
        while offset < used {
            let mut decode = std::mem::zeroed();
            hde64_disasm(backup.as_ptr().add(offset), &mut decode);
            let width = decode.len as usize;

            let mut copied = false;

            #[cfg(target_pointer_width = "64")]
            {
                if (decode.modrm & 0xc7) == 0x05 && decode.opcode == 0x8b {
                    let destiny = (symbol.add(offset).add(width) as isize + decode.disp.disp32 as i32 as isize) as usize;
                    let reg = (decode.rex_r << 3) | decode.modrm_reg;
                    push_pointer(&mut current, destiny);
                    write_pop(&mut current, reg);
                    write_move64(&mut current, reg, reg);
                    copied = true;
                }
            }

            if !copied {
                if backup[offset] == 0xe8 {
                    let relative = ptr::read_unaligned(backup.as_ptr().add(offset + 1) as *const i32);
                    if relative == 0 {
                        push_pointer(&mut current, symbol.add(offset).add(decode.len as usize) as usize);
                    } else {
                        write_u8(&mut current, 0xe8);
                        write_i32(&mut current, size_of_skip() as i32);
                        let destiny = symbol.add(offset).add(decode.len as usize).offset(relative as isize);
                        let current_pos = current as usize + size_of_skip();
                        write_skip(&mut current, size_of_jump(destiny as usize, current_pos) as isize);
                        write_jump(&mut current, destiny as usize);
                    }
                } else if backup[offset] == 0xeb {
                    write_jump(&mut current, (symbol.add(offset).add(decode.len as usize) as isize + *backup.as_ptr().add(offset + 1) as i8 as isize) as usize);
                } else if backup[offset] == 0xe9 {
                    write_jump(&mut current, (symbol.add(offset).add(decode.len as usize) as isize + ptr::read_unaligned(backup.as_ptr().add(offset + 1) as *const i32) as isize) as usize);
                } else if backup[offset] == 0xe3 || (backup[offset] & 0xf0) == 0x70 {
                    write_u8(&mut current, backup[offset]);
                    write_u8(&mut current, 2);
                    write_u8(&mut current, 0xeb);
                    let destiny = (symbol.add(offset).add(decode.len as usize) as isize + *backup.as_ptr().add(offset + 1) as i8 as isize) as usize;
                    let current_pos = current as usize + 1;
                    write_u8(&mut current, size_of_jump(destiny, current_pos) as u8);
                    write_jump(&mut current, destiny);
                } else {
                    write_bytes(&mut current, backup.as_ptr().add(offset), width);
                }
            }

            offset += width;
        }

        write_jump(&mut current, symbol.add(used) as usize);

        make_executable(buffer, length)?;
        *result = buffer;
    }

    {
        let _code = ProtectedMemory::new(symbol, used)?;
        let mut current = symbol;
        write_jump(&mut current, target);
        for _ in 0..blank {
            write_u8(&mut current, 0x90);
        }
    }

    Ok(used)
}
