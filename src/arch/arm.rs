use crate::disasm::arm_decoder::is_arm_pc_relative;
use crate::error::{Result, SubstrateError};
use crate::hook::memory::{allocate_trampoline, make_executable, ProtectedMemory};
use std::ptr;

#[repr(u32)]
#[allow(dead_code)]
enum AReg {
    R0 = 0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15,
}

const A_SP: u32 = 13;
const A_LR: u32 = 14;
const A_PC: u32 = 15;
const A_R0: u32 = 0;
const A_R1: u32 = 1;

fn a_ldr_rd_rn_im(rd: u32, rn: u32, im: i32) -> u32 {
    0xe5100000 | (if im < 0 { 0 } else { 1 << 23 }) | ((rn) << 16) | ((rd) << 12) | im.abs() as u32
}

fn a_stmdb_sp_rs(rs: u32) -> u32 {
    0xe9200000 | (A_SP << 16) | rs
}

fn a_ldmia_sp_rs(rs: u32) -> u32 {
    0xe8b00000 | (A_SP << 16) | rs
}

pub unsafe fn hook_function_arm(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullPointer);
    }

    let area = symbol as *mut u32;
    let arm = area;
    let used = 8;

    let backup = [*arm, *arm.add(1)];

    if !result.is_null() {
        if backup[0] == a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8) {
            *result = backup[1] as *mut u8;
            return Ok(4);
        }

        let mut length = used;
        for offset in 0..(used / 4) {
            if is_arm_pc_relative(backup[offset]) {
                if (backup[offset] & 0x02000000) == 0
                    || (backup[offset] & 0x0000f000) >> 12 != (backup[offset] & 0x0000000f)
                {
                    length += 2 * 4;
                } else {
                    length += 4 * 4;
                }
            }
        }

        length += 2 * 4;

        let buffer = allocate_trampoline(length)? as *mut u32;

        let mut start = 0;
        let mut end = length / 4;

        for offset in 0..(used / 4) {
            if is_arm_pc_relative(backup[offset]) {
                let value = backup[offset];

                let rm = value & 0xf;
                let rd = (value >> 12) & 0xf;
                let rn = (value >> 16) & 0xf;
                let mode = (value >> 25) & 0x1;

                let (copy_rn, guard) = if mode == 0 || rd != rm {
                    (rd, false)
                } else {
                    (if rm != A_R0 { A_R0 } else { A_R1 }, true)
                };

                if guard {
                    *buffer.add(start) = a_stmdb_sp_rs(1 << copy_rn);
                    start += 1;
                }

                *buffer.add(start) = a_ldr_rd_rn_im(copy_rn, A_PC, ((end - 1 - start) * 4 - 8) as i32);
                *buffer.add(start + 1) = (value & !0x000f0000) | (copy_rn << 16);
                start += 2;

                if guard {
                    *buffer.add(start) = a_ldmia_sp_rs(1 << copy_rn);
                    start += 1;
                }

                end -= 1;
                *buffer.offset(end as isize) = (area as usize + offset * 4 + 8) as u32;
            } else {
                *buffer.add(start) = backup[offset];
                start += 1;
            }
        }

        *buffer.add(start) = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
        *buffer.add(start + 1) = (area as usize + used) as u32;

        make_executable(buffer as *mut u8, length)?;
        *result = buffer as *mut u8;
    }

    {
        let _code = ProtectedMemory::new(symbol, used)?;
        *arm = a_ldr_rd_rn_im(A_PC, A_PC, 4 - 8);
        *arm.add(1) = replace as u32;
    }

    Ok(used)
}
