use crate::disasm::arm_decoder::*;
use crate::error::{Result, SubstrateError};
use crate::hook::memory::{allocate_trampoline, make_executable, ProtectedMemory};
use std::ptr;

const A_PC: u32 = 15;
const A_LR: u32 = 14;
const A_R6: u32 = 6;
const A_R7: u32 = 7;
const A_AL: u32 = 14;

fn t_label(l: usize, r: usize) -> i32 {
    ((r as isize - l as isize) * 2 - 4 + if l % 2 == 0 { 0 } else { 2 }) as i32
}

fn t_bx(rm: u32) -> u16 {
    0x4700 | ((rm << 3) as u16)
}

fn t_nop() -> u16 {
    0x46c0
}

fn t_push_r(r: u32) -> u16 {
    (0xb400 | (((r & (1 << A_LR)) >> A_LR) << 8) | (r & 0xff)) as u16
}

fn t_pop_r(r: u32) -> u16 {
    (0xbc00 | (((r & (1 << A_PC)) >> A_PC) << 8) | (r & 0xff)) as u16
}

fn t_mov_rd_rm(rd: u32, rm: u32) -> u16 {
    (0x4600 | (((rd & 0x8) >> 3) << 7) | (((rm & 0x8) >> 3) << 6) | ((rm & 0x7) << 3) | (rd & 0x7)) as u16
}

fn t_ldr_rd_pc_im_4(rd: u32, im: u32) -> u16 {
    (0x4800 | ((rd << 8) | (im & 0xff))) as u16
}

fn t_ldr_rd_rn_im_4(rd: u32, rn: u32, im: u32) -> u16 {
    (0x6800 | (((im & 0x1f) << 6) | ((rn << 3) | rd))) as u16
}

fn t_add_rd_rm(rd: u32, rm: u32) -> u16 {
    (0x4400 | (((rd & 0x8) >> 3) << 7) | (((rm & 0x8) >> 3) << 6) | ((rm & 0x7) << 3) | (rd & 0x7)) as u16
}

fn t_blx(rm: u32) -> u16 {
    0x4780 | ((rm << 3) as u16)
}

fn t_b_im(cond: u32, im: i32) -> u16 {
    if cond == A_AL {
        (0xe000 | ((im >> 1) & 0x7ff)) as u16
    } else {
        (0xd000 | ((cond << 8) | (((im >> 1) as u32) & 0xff))) as u16
    }
}

fn t_cbz_rn_im(op: u32, rn: u32, im: i32) -> u16 {
    (0xb100 | ((op << 11) | (((im as u32 & 0x40) >> 6) << 9) | (((im as u32 & 0x3e) >> 1) << 3) | rn)) as u16
}

fn t1_mrs_rd_apsr(_rd: u32) -> u16 {
    0xf3ef
}

fn t2_mrs_rd_apsr(_rd: u32) -> u16 {
    0x8000 | ((_rd << 8) as u16)
}

fn t1_msr_apsr_nzcvqg_rn(rn: u32) -> u16 {
    (0xf380 | rn) as u16
}

fn t2_msr_apsr_nzcvqg_rn(_rn: u32) -> u16 {
    0x8c00
}

fn t_msr_apsr_nzcvqg_rn(rn: u32) -> u32 {
    ((t2_msr_apsr_nzcvqg_rn(rn) as u32) << 16) | (t1_msr_apsr_nzcvqg_rn(rn) as u32)
}

fn t1_ldr_rt_rn_im(_rt: u32, rn: u32, im: i32) -> u16 {
    (0xf850 | (if im < 0 { 0 } else { 1 << 7 }) | rn) as u16
}

fn t2_ldr_rt_rn_im(rt: u32, _rn: u32, im: i32) -> u16 {
    ((rt << 12) | im.abs() as u32) as u16
}

fn a_ldr_rd_rn_im(rd: u32, rn: u32, im: i32) -> u32 {
    0xe5100000 | (if im < 0 { 0 } else { 1 << 23 }) | ((rn) << 16) | ((rd) << 12) | im.abs() as u32
}

pub unsafe fn hook_function_thumb(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullPointer);
    }

    let area = symbol as *mut u16;
    let align = if (area as usize & 0x2) == 0 { 0 } else { 1 };
    let thumb = area.add(align);
    let arm = thumb.add(2) as *mut u32;
    let trail = arm.add(2) as *mut u16;

    if (align == 0 || *area == t_nop())
        && *thumb == t_bx(A_PC)
        && *thumb.add(1) == t_nop()
        && *arm == a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8)
    {
        if !result.is_null() {
            *result = *arm.add(1) as *mut u8;
        }

        let _code = ProtectedMemory::new(arm.add(1) as *mut u8, 4)?;
        *arm.add(1) = replace as u32;

        return Ok(4);
    }

    let required = ((trail as usize - area as usize) / 2) * 2;
    let mut used = 0;

    while used < required {
        used += get_thumb_instruction_width(area.add(used / 2) as *const u8);
    }
    used = (used + 2 - 1) / 2 * 2;

    let blank = (used - required) / 2;

    let mut backup = vec![0u16; used / 2];
    ptr::copy_nonoverlapping(area, backup.as_mut_ptr(), used / 2);

    if !result.is_null() {
        let mut length = used;

        for offset in 0..(used / 2) {
            if is_thumb_pc_relative_ldr(backup[offset]) {
                length += 3 * 2;
            } else if is_thumb_pc_relative_b(backup[offset]) {
                length += 6 * 2;
            } else if is_thumb2_pc_relative_b(&backup[offset..]) {
                length += 5 * 2;
            } else if is_thumb_pc_relative_bl(&backup[offset..]) {
                length += 5 * 2;
            } else if is_thumb_pc_relative_cbz(backup[offset]) {
                length += 16 * 2;
            } else if is_thumb_pc_relative_ldrw(backup[offset]) {
                length += 4 * 2;
            } else if is_thumb_pc_relative_add(backup[offset]) {
                length += 6 * 2;
            }
        }

        let pad = if (length & 0x2) == 0 { 0 } else { 1 };
        length += (pad + 2) * 2 + 2 * 4;

        let buffer = allocate_trampoline(length)? as *mut u16;

        let mut start = pad;
        let mut end = length / 2;
        let mut trailer = buffer.add(end) as *mut u32;

        let mut offset = 0;
        while offset < used / 2 {
            if is_thumb_pc_relative_ldr(backup[offset]) {
                let immediate = (backup[offset] & 0xff) as u32;
                let rd = ((backup[offset] >> 8) & 0x7) as u32;

                *buffer.add(start) = t_ldr_rd_pc_im_4(rd, (t_label(start, end - 2) / 4) as u32);
                *buffer.add(start + 1) = t_ldr_rd_rn_im_4(rd, rd, 0);

                trailer = trailer.sub(1);
                *trailer = ((area.add(offset) as usize + 4) & !0x2) as u32 + immediate * 4;

                start += 2;
                end -= 2;
            } else if is_thumb_pc_relative_b(backup[offset]) {
                let imm8 = (backup[offset] & 0xff) as i32;
                let cond = ((backup[offset] >> 8) & 0xf) as u32;

                let mut jump = imm8 << 1;
                jump |= 1;
                jump <<= 23;
                jump >>= 23;

                *buffer.add(start) = t_b_im(cond, (end as i32 - 6 - start as i32) * 2 - 4);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as usize + 4 + jump as usize) as u32;
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((t_nop() as u32) << 16) | (t_bx(A_PC) as u32);

                start += 1;
                end -= 6;
            } else if is_thumb2_pc_relative_b(&backup[offset..]) {
                let bits0 = backup[offset];
                let bits1 = backup[offset + 1];

                let imm6 = (bits0 & 0x3f) as i32;
                let cond = ((bits0 >> 6) & 0xf) as u32;
                let s = ((bits0 >> 10) & 0x1) as i32;

                let imm11 = (bits1 & 0x7ff) as i32;
                let j2 = ((bits1 >> 11) & 0x1) as i32;
                let a = ((bits1 >> 12) & 0x1) as i32;
                let j1 = ((bits1 >> 13) & 0x1) as i32;

                let mut jump = 1;
                jump |= imm11 << 1;
                jump |= imm6 << 12;

                if a != 0 {
                    jump |= s << 24;
                    jump |= (!(s ^ j1) & 0x1) << 23;
                    jump |= (!(s ^ j2) & 0x1) << 22;
                    jump |= (cond as i32) << 18;
                    jump <<= 7;
                    jump >>= 7;
                } else {
                    jump |= s << 20;
                    jump |= j2 << 19;
                    jump |= j1 << 18;
                    jump <<= 11;
                    jump >>= 11;
                }

                *buffer.add(start) = t_b_im(if a != 0 { A_AL } else { cond }, (end as i32 - 6 - start as i32) * 2 - 4);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as usize + 4 + jump as usize) as u32;
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((t_nop() as u32) << 16) | (t_bx(A_PC) as u32);

                offset += 1;
                start += 1;
                end -= 6;
            } else if is_thumb_pc_relative_bl(&backup[offset..]) {
                let bits0 = backup[offset];
                let bits1 = backup[offset + 1];

                let immediate = (bits0 & 0x3ff) as i32;
                let s = ((bits0 >> 10) & 0x1) as i32;

                let immediate2 = (bits1 & 0x7ff) as i32;
                let j2 = ((bits1 >> 11) & 0x1) as i32;
                let x = ((bits1 >> 12) & 0x1) as i32;
                let j1 = ((bits1 >> 13) & 0x1) as i32;

                let mut jump = 0;
                jump |= s << 24;
                jump |= (!(s ^ j1) & 0x1) << 23;
                jump |= (!(s ^ j2) & 0x1) << 22;
                jump |= immediate << 12;
                jump |= immediate2 << 1;
                jump |= x;
                jump <<= 7;
                jump >>= 7;

                *buffer.add(start) = t_push_r(1 << A_R7);
                *buffer.add(start + 1) = t_ldr_rd_pc_im_4(A_R7, (((end - 2 - (start + 1)) * 2 - 4 + 2) / 4) as u32);
                *buffer.add(start + 2) = t_mov_rd_rm(A_LR, A_R7);
                *buffer.add(start + 3) = t_pop_r(1 << A_R7);
                *buffer.add(start + 4) = t_blx(A_LR);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as usize + 4 + jump as usize) as u32;

                offset += 1;
                start += 5;
                end -= 2;
            } else if is_thumb_pc_relative_cbz(backup[offset]) {
                let rn = (backup[offset] & 0x7) as u32;
                let immediate = ((backup[offset] >> 3) & 0x1f) as i32;
                let i = ((backup[offset] >> 9) & 0x1) as i32;
                let op = ((backup[offset] >> 11) & 0x1) as u32;

                let mut jump = 1;
                jump |= i << 6;
                jump |= immediate << 1;

                let rt = if rn == A_R7 { A_R6 } else { A_R7 };

                *buffer.add(start) = t_push_r(1 << rt);
                *buffer.add(start + 1) = t1_mrs_rd_apsr(rt);
                *buffer.add(start + 2) = t2_mrs_rd_apsr(rt);
                *buffer.add(start + 3) = t_cbz_rn_im(op, rn, (end as i32 - 10 - (start + 3) as i32) * 2 - 4);
                *buffer.add(start + 4) = t1_msr_apsr_nzcvqg_rn(rt);
                *buffer.add(start + 5) = t2_msr_apsr_nzcvqg_rn(rt);
                *buffer.add(start + 6) = t_pop_r(1 << rt);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as usize + 4 + jump as usize) as u32;
                trailer = trailer.sub(1);
                *trailer = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
                trailer = trailer.sub(1);
                *trailer = ((t_nop() as u32) << 16) | (t_bx(A_PC) as u32);
                trailer = trailer.sub(1);
                *trailer = ((t_nop() as u32) << 16) | (t_pop_r(1 << rt) as u32);
                trailer = trailer.sub(1);
                *trailer = t_msr_apsr_nzcvqg_rn(rt);

                start += 7;
                end -= 10;
            } else if is_thumb_pc_relative_ldrw(backup[offset]) {
                let bits0 = backup[offset];
                let bits1 = backup[offset + 1];

                let u = ((bits0 >> 7) & 0x1) as i32;
                let immediate = (bits1 & 0xfff) as i32;
                let rt = ((bits1 >> 12) & 0xf) as u32;

                *buffer.add(start) = t1_ldr_rt_rn_im(rt, A_PC, t_label(start, end - 2));
                *buffer.add(start + 1) = t2_ldr_rt_rn_im(rt, A_PC, t_label(start, end - 2));
                *buffer.add(start + 2) = t1_ldr_rt_rn_im(rt, rt, 0);
                *buffer.add(start + 3) = t2_ldr_rt_rn_im(rt, rt, 0);

                trailer = trailer.sub(1);
                *trailer = (((area.add(offset) as usize + 4) & !0x2) as i32 + if u == 0 { -immediate } else { immediate }) as u32;

                offset += 1;
                start += 4;
                end -= 2;
            } else if is_thumb_pc_relative_add(backup[offset]) {
                let rd = (backup[offset] & 0x7) as u32;
                let _rm = ((backup[offset] >> 3) & 0x7) as u32;
                let h1 = ((backup[offset] >> 7) & 0x1) as u32;

                if h1 != 0 {
                    return Err(SubstrateError::HookFailed("PC-relative add with h1 set".to_string()));
                }

                let rt = if rd == A_R7 { A_R6 } else { A_R7 };

                *buffer.add(start) = t_push_r(1 << rt);
                *buffer.add(start + 1) = t_mov_rd_rm(rt, (h1 << 3) | rd);
                *buffer.add(start + 2) = t_ldr_rd_pc_im_4(rd, (t_label(start + 2, end - 2) / 4) as u32);
                *buffer.add(start + 3) = t_add_rd_rm((h1 << 3) | rd, rt);
                *buffer.add(start + 4) = t_pop_r(1 << rt);

                trailer = trailer.sub(1);
                *trailer = (area.add(offset) as usize + 4) as u32;

                start += 5;
                end -= 2;
            } else if is_thumb_32bit(backup[offset]) {
                *buffer.add(start) = backup[offset];
                *buffer.add(start + 1) = backup[offset + 1];
                start += 2;
                offset += 1;
            } else {
                *buffer.add(start) = backup[offset];
                start += 1;
            }

            offset += 1;
        }

        *buffer.add(start) = t_bx(A_PC);
        *buffer.add(start + 1) = t_nop();

        let transfer = buffer.add(start + 2) as *mut u32;
        *transfer = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
        *transfer.add(1) = (area.add(used / 2) as usize + 1) as u32;

        make_executable(buffer as *mut u8, length)?;
        *result = buffer.add(pad) as *mut u8;
        *result = (*result as usize + 1) as *mut u8;
    }

    {
        let _code = ProtectedMemory::new(area as *mut u8, used)?;

        if align != 0 {
            *area = t_nop();
        }

        *thumb = t_bx(A_PC);
        *thumb.add(1) = t_nop();
        *arm = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
        *arm.add(1) = replace as u32;

        for i in 0..blank {
            *trail.add(i) = t_nop();
        }
    }

    Ok(used)
}
