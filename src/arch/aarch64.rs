use crate::error::{Result, SubstrateError};
use crate::hook::memory::{allocate_trampoline, ProtectedMemory};
use std::ptr;
use std::sync::atomic::{AtomicI32, Ordering};

const A64_MAX_INSTRUCTIONS: usize = 5;
const A64_MAX_REFERENCES: usize = A64_MAX_INSTRUCTIONS * 2;
const A64_NOP: u32 = 0xd503201f;
const A64_MAX_BACKUPS: usize = 256;

#[repr(C)]
struct FixInfo {
    bp: *mut u32,
    ls: u32,
    ad: u32,
}

#[repr(C)]
struct InsnsInfo {
    insp: *mut u32,
    fmap: [FixInfo; A64_MAX_REFERENCES],
}

struct Context {
    basep: i64,
    endp: i64,
    dat: [InsnsInfo; A64_MAX_INSTRUCTIONS],
}

impl Context {
    fn new(inp: *mut u32, count: i32) -> Self {
        let basep = inp as i64;
        let endp = unsafe { inp.add(count as usize) } as i64;

        Self {
            basep,
            endp,
            dat: unsafe { std::mem::zeroed() },
        }
    }

    fn is_in_fixing_range(&self, absolute_addr: i64) -> bool {
        absolute_addr >= self.basep && absolute_addr < self.endp
    }

    fn get_ref_ins_index(&self, absolute_addr: i64) -> isize {
        ((absolute_addr - self.basep) / 4) as isize
    }

    fn get_and_set_current_index(&mut self, inp: *mut u32, outp: *mut u32) -> isize {
        let current_idx = self.get_ref_ins_index(inp as i64);
        self.dat[current_idx as usize].insp = outp;
        current_idx
    }

    fn reset_current_ins(&mut self, idx: isize, outp: *mut u32) {
        self.dat[idx as usize].insp = outp;
    }

    fn insert_fix_map(&mut self, idx: isize, bp: *mut u32, ls: u32, ad: u32) {
        for f in &mut self.dat[idx as usize].fmap {
            if f.bp.is_null() {
                f.bp = bp;
                f.ls = ls;
                f.ad = ad;
                return;
            }
        }
    }

    fn process_fix_map(&mut self, idx: isize) {
        for f in &mut self.dat[idx as usize].fmap {
            if f.bp.is_null() {
                break;
            }
            unsafe {
                let offset = ((self.dat[idx as usize].insp as i64 - f.bp as i64) >> 2) as i32;
                *f.bp |= ((offset << f.ls) as u32) & f.ad;
                f.bp = ptr::null_mut();
            }
        }
    }
}

unsafe fn fix_branch_imm(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctxp: &mut Context,
) -> bool {
    const MASK: u32 = 0xfc000000;
    const RMASK: u32 = 0x03ffffff;
    const OP_B: u32 = 0x14000000;
    const OP_BL: u32 = 0x94000000;

    let ins = **inpp;
    let opc = ins & MASK;

    if opc == OP_B || opc == OP_BL {
        let current_idx = ctxp.get_and_set_current_index(*inpp, *outpp);
        let absolute_addr = (*inpp as i64) + (((ins << 6) as i32 >> 4) as i64);
        let mut new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
        let special_fix_type = ctxp.is_in_fixing_range(absolute_addr);

        if !special_fix_type && new_pc_offset.abs() >= (RMASK as i64 >> 1) {
            let b_aligned = ((*outpp.add(2) as usize) & 7) == 0;

            if opc == OP_B {
                if !b_aligned {
                    **outpp = A64_NOP;
                    *outpp = outpp.add(1);
                    ctxp.reset_current_ins(current_idx, *outpp);
                }
                (**outpp) = 0x58000051;
                outpp.add(1).write(0xd61f0220);
                ptr::copy_nonoverlapping(
                    &absolute_addr as *const i64 as *const u32,
                    outpp.add(2),
                    2,
                );
                *outpp = outpp.add(4);
            } else {
                if b_aligned {
                    **outpp = A64_NOP;
                    *outpp = outpp.add(1);
                    ctxp.reset_current_ins(current_idx, *outpp);
                }
                **outpp = 0x58000071;
                outpp.add(1).write(0x1000009e);
                outpp.add(2).write(0xd61f0220);
                ptr::copy_nonoverlapping(
                    &absolute_addr as *const i64 as *const u32,
                    outpp.add(3),
                    2,
                );
                *outpp = outpp.add(5);
            }
        } else {
            if special_fix_type {
                let ref_idx = ctxp.get_ref_ins_index(absolute_addr);
                if ref_idx <= current_idx {
                    new_pc_offset = (ctxp.dat[ref_idx as usize].insp as i64 - *outpp as i64) >> 2;
                } else {
                    ctxp.insert_fix_map(ref_idx, *outpp, 0, RMASK);
                    new_pc_offset = 0;
                }
            }

            **outpp = opc | ((new_pc_offset as u32) & !MASK);
            *outpp = outpp.add(1);
        }

        *inpp = inpp.add(1);
        ctxp.process_fix_map(current_idx);
        return true;
    }

    false
}

unsafe fn fix_cond_comp_test_branch(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctxp: &mut Context,
) -> bool {
    const LSB: u32 = 5;
    const LMASK01: u32 = 0xff00001f;
    const MASK0: u32 = 0xff000010;
    const OP_BC: u32 = 0x54000000;
    const MASK1: u32 = 0x7f000000;
    const OP_CBZ: u32 = 0x34000000;
    const OP_CBNZ: u32 = 0x35000000;
    const LMASK2: u32 = 0xfff8001f;
    const MASK2: u32 = 0x7f000000;
    const OP_TBZ: u32 = 0x36000000;
    const OP_TBNZ: u32 = 0x37000000;

    let ins = **inpp;
    let mut lmask = LMASK01;

    if (ins & MASK0) != OP_BC {
        let mut opc = ins & MASK1;
        if opc != OP_CBZ && opc != OP_CBNZ {
            opc = ins & MASK2;
            if opc != OP_TBZ && opc != OP_TBNZ {
                return false;
            }
            lmask = LMASK2;
        }
    }

    let current_idx = ctxp.get_and_set_current_index(*inpp, *outpp);
    let absolute_addr = (*inpp as i64) + (((ins & !lmask) >> (LSB - 2)) as i64);
    let mut new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
    let special_fix_type = ctxp.is_in_fixing_range(absolute_addr);

    if !special_fix_type && new_pc_offset.abs() >= (!lmask >> (LSB + 1)) as i64 {
        if ((*outpp.add(4) as usize) & 7) != 0 {
            **outpp = A64_NOP;
            *outpp = outpp.add(1);
            ctxp.reset_current_ins(current_idx, *outpp);
        }
        **outpp = (((8 >> 2) << LSB) & !lmask) | (ins & lmask);
        outpp.add(1).write(0x14000005);
        outpp.add(2).write(0x58000051);
        outpp.add(3).write(0xd61f0220);
        ptr::copy_nonoverlapping(
            &absolute_addr as *const i64 as *const u32,
            outpp.add(4),
            2,
        );
        *outpp = outpp.add(6);
    } else {
        if special_fix_type {
            let ref_idx = ctxp.get_ref_ins_index(absolute_addr);
            if ref_idx <= current_idx {
                new_pc_offset = (ctxp.dat[ref_idx as usize].insp as i64 - *outpp as i64) >> 2;
            } else {
                ctxp.insert_fix_map(ref_idx, *outpp, LSB, !lmask);
                new_pc_offset = 0;
            }
        }

        **outpp = (((new_pc_offset as u32) << LSB) & !lmask) | (ins & lmask);
        *outpp = outpp.add(1);
    }

    *inpp = inpp.add(1);
    ctxp.process_fix_map(current_idx);
    true
}

unsafe fn fix_loadlit(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctxp: &mut Context,
) -> bool {
    let ins = **inpp;

    if (ins & 0xff000000) == 0xd8000000 {
        let idx = ctxp.get_and_set_current_index(*inpp, *outpp);
        ctxp.process_fix_map(idx);
        *inpp = inpp.add(1);
        return true;
    }

    const MSB: u32 = 8;
    const LSB: u32 = 5;
    const MASK_30: u32 = 0x40000000;
    const MASK_31: u32 = 0x80000000;
    const LMASK: u32 = 0xff00001f;
    const MASK_LDR: u32 = 0xbf000000;
    const OP_LDR: u32 = 0x18000000;
    const MASK_LDRV: u32 = 0x3f000000;
    const OP_LDRV: u32 = 0x1c000000;
    const MASK_LDRSW: u32 = 0xff000000;
    const OP_LDRSW: u32 = 0x98000000;

    let mut mask = MASK_LDR;
    let mut faligned = if (ins & MASK_30) != 0 { 7 } else { 3 };

    if (ins & MASK_LDR) != OP_LDR {
        mask = MASK_LDRV;
        if faligned != 7 {
            faligned = if (ins & MASK_31) != 0 { 15 } else { 3 };
        }
        if (ins & MASK_LDRV) != OP_LDRV {
            if (ins & MASK_LDRSW) != OP_LDRSW {
                return false;
            }
            mask = MASK_LDRSW;
            faligned = 7;
        }
    }

    let current_idx = ctxp.get_and_set_current_index(*inpp, *outpp);
    let absolute_addr = (*inpp as i64) + (((((ins << MSB) as i32) >> (MSB + LSB - 2)) & !3) as i64);
    let new_pc_offset = (absolute_addr - *outpp as i64) >> 2;
    let special_fix_type = ctxp.is_in_fixing_range(absolute_addr);

    if special_fix_type || (new_pc_offset.abs() + ((faligned + 1 - 4) / 4) as i64) >= (!LMASK >> (LSB + 1)) as i64 {
        while ((*outpp.add(2) as usize) & faligned) != 0 {
            **outpp = A64_NOP;
            *outpp = outpp.add(1);
        }
        ctxp.reset_current_ins(current_idx, *outpp);

        let ns = (faligned + 1) / 4;
        **outpp = (((8 >> 2) << LSB) & !mask) | (ins & LMASK);
        outpp.add(1).write((0x14000001 + ns) as u32);
        ptr::copy_nonoverlapping(
            absolute_addr as *const u32,
            outpp.add(2),
            ns as usize,
        );
        *outpp = outpp.add(2 + ns as usize);
    } else {
        let mut new_offset = new_pc_offset;
        let mut faligned_shifted = faligned >> 2;
        while (new_offset & (faligned_shifted as i64)) != 0 {
            **outpp = A64_NOP;
            *outpp = outpp.add(1);
            new_offset = (absolute_addr - *outpp as i64) >> 2;
        }
        ctxp.reset_current_ins(current_idx, *outpp);

        **outpp = (((new_offset as u32) << LSB) & !mask) | (ins & LMASK);
        *outpp = outpp.add(1);
    }

    *inpp = inpp.add(1);
    ctxp.process_fix_map(current_idx);
    true
}

unsafe fn fix_pcreladdr(
    inpp: &mut *mut u32,
    outpp: &mut *mut u32,
    ctxp: &mut Context,
) -> bool {
    const MSB: u32 = 8;
    const LSB: u32 = 5;
    const MASK: u32 = 0x9f000000;
    const RMASK: u32 = 0x0000001f;
    const LMASK: u32 = 0xff00001f;
    const FMASK: u32 = 0x00ffffff;
    const MAX_VAL: u32 = 0x001fffff;
    const OP_ADR: u32 = 0x10000000;
    const OP_ADRP: u32 = 0x90000000;

    let ins = **inpp;

    match ins & MASK {
        OP_ADR => {
            let current_idx = ctxp.get_and_set_current_index(*inpp, *outpp);
            let lsb_bytes = ((ins << 1) >> 30) as i64;
            let absolute_addr = (*inpp as i64) + (((((ins << MSB) as i32) >> (MSB + LSB - 2)) & !3) as i64 | lsb_bytes);
            let mut new_pc_offset = absolute_addr - *outpp as i64;
            let special_fix_type = ctxp.is_in_fixing_range(absolute_addr);

            if !special_fix_type && new_pc_offset.abs() >= (MAX_VAL as i64 >> 1) {
                if ((*outpp.add(2) as usize) & 7) != 0 {
                    **outpp = A64_NOP;
                    *outpp = outpp.add(1);
                    ctxp.reset_current_ins(current_idx, *outpp);
                }

                **outpp = 0x58000000 | (((8 >> 2) << LSB) & !MASK) | (ins & RMASK);
                outpp.add(1).write(0x14000003);
                ptr::copy_nonoverlapping(
                    &absolute_addr as *const i64 as *const u32,
                    outpp.add(2),
                    2,
                );
                *outpp = outpp.add(4);
            } else {
                if special_fix_type {
                    let ref_idx = ctxp.get_ref_ins_index(absolute_addr & !3);
                    if ref_idx <= current_idx {
                        new_pc_offset = ctxp.dat[ref_idx as usize].insp as i64 - *outpp as i64;
                    } else {
                        ctxp.insert_fix_map(ref_idx, *outpp, LSB, FMASK);
                        new_pc_offset = 0;
                    }
                }

                **outpp = (((new_pc_offset as u32) << (LSB - 2)) & FMASK) | (ins & LMASK);
                *outpp = outpp.add(1);
            }

            *inpp = inpp.add(1);
            ctxp.process_fix_map(current_idx);
            true
        }
        OP_ADRP => {
            let current_idx = ctxp.get_and_set_current_index(*inpp, *outpp);
            let lsb_bytes = ((ins << 1) >> 30) as i32;
            let absolute_addr = ((*inpp as i64) & !0xfff) + (((((((ins << MSB) as i32) >> (MSB + LSB - 2)) & !3) | lsb_bytes) as i64) << 12);

            if ctxp.is_in_fixing_range(absolute_addr) {
                **outpp = ins;
                *outpp = outpp.add(1);
            } else {
                if ((*outpp.add(2) as usize) & 7) != 0 {
                    **outpp = A64_NOP;
                    *outpp = outpp.add(1);
                    ctxp.reset_current_ins(current_idx, *outpp);
                }

                **outpp = 0x58000000 | (((8 >> 2) << LSB) & !MASK) | (ins & RMASK);
                outpp.add(1).write(0x14000003);
                ptr::copy_nonoverlapping(
                    &absolute_addr as *const i64 as *const u32,
                    outpp.add(2),
                    2,
                );
                *outpp = outpp.add(4);
            }

            *inpp = inpp.add(1);
            ctxp.process_fix_map(current_idx);
            true
        }
        _ => false,
    }
}

unsafe fn fix_instructions(inp: *mut u32, count: i32, outp: *mut u32) {
    let mut ctx = Context::new(inp, count);
    let outp_base = outp;
    let mut inp_cur = inp;
    let mut outp_cur = outp;
    let mut remaining = count;

    while remaining > 0 {
        if fix_branch_imm(&mut inp_cur, &mut outp_cur, &mut ctx) {
            remaining -= 1;
            continue;
        }
        if fix_cond_comp_test_branch(&mut inp_cur, &mut outp_cur, &mut ctx) {
            remaining -= 1;
            continue;
        }
        if fix_loadlit(&mut inp_cur, &mut outp_cur, &mut ctx) {
            remaining -= 1;
            continue;
        }
        if fix_pcreladdr(&mut inp_cur, &mut outp_cur, &mut ctx) {
            remaining -= 1;
            continue;
        }

        let idx = ctx.get_and_set_current_index(inp_cur, outp_cur);
        ctx.process_fix_map(idx);
        *outp_cur = *inp_cur;
        inp_cur = inp_cur.add(1);
        outp_cur = outp_cur.add(1);
        remaining -= 1;
    }

    let callback = inp_cur;
    let mut pc_offset = (callback as i64 - outp_cur as i64) >> 2;

    if pc_offset.abs() >= (0x03ffffff >> 1) {
        if ((outp_cur.add(2) as usize) & 7) != 0 {
            *outp_cur = A64_NOP;
            outp_cur = outp_cur.add(1);
        }
        *outp_cur = 0x58000051;
        *outp_cur.add(1) = 0xd61f0220;
        ptr::copy_nonoverlapping(
            &callback as *const *mut u32 as *const u32,
            outp_cur.add(2),
            2,
        );
        outp_cur = outp_cur.add(4);
    } else {
        *outp_cur = 0x14000000 | ((pc_offset & 0x03ffffff) as u32);
        outp_cur = outp_cur.add(1);
    }

    let total = (outp_cur as usize - outp_base as usize) / 4;
    clear_cache(outp_base as *mut u8, total * 4);
}

unsafe fn clear_cache(ptr: *mut u8, size: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" {
            fn __clear_cache(start: *mut u8, end: *mut u8);
        }
        __clear_cache(ptr, ptr.add(size));
    }
}

static TRAMPOLINE_INDEX: AtomicI32 = AtomicI32::new(-1);
static mut INSNS_POOL: [[u32; A64_MAX_INSTRUCTIONS * 10]; A64_MAX_BACKUPS] =
    [[0; A64_MAX_INSTRUCTIONS * 10]; A64_MAX_BACKUPS];

unsafe fn fast_allocate_trampoline() -> Option<*mut u32> {
    let i = TRAMPOLINE_INDEX.fetch_add(1, Ordering::SeqCst) + 1;
    if i >= 0 && i < A64_MAX_BACKUPS as i32 {
        Some(INSNS_POOL[i as usize].as_mut_ptr())
    } else {
        None
    }
}

pub unsafe fn hook_function_aarch64(
    symbol: *mut u8,
    replace: *mut u8,
    result: *mut *mut u8,
) -> Result<usize> {
    if symbol.is_null() {
        return Err(SubstrateError::NullPointer);
    }

    static POOL_INIT: std::sync::Once = std::sync::Once::new();
    POOL_INIT.call_once(|| {
        let _ = ProtectedMemory::new(
            INSNS_POOL.as_mut_ptr() as *mut u8,
            std::mem::size_of_val(&INSNS_POOL),
        );
    });

    let trampoline = if !result.is_null() {
        match fast_allocate_trampoline() {
            Some(t) => t as *mut u8,
            None => return Err(SubstrateError::HookFailed("Failed to allocate trampoline".to_string())),
        }
    } else {
        ptr::null_mut()
    };

    let original = symbol as *mut u32;
    let pc_offset = (replace as i64 - symbol as i64) >> 2;

    if pc_offset.abs() >= (0x03ffffff >> 1) {
        let count = if (((original.add(2) as usize) & 7) != 0) { 5 } else { 4 };

        if !trampoline.is_null() {
            fix_instructions(original, count, trampoline as *mut u32);
        }

        let _code = ProtectedMemory::new(original as *mut u8, 5 * 4)?;

        if count == 5 {
            *original = A64_NOP;
            let target = original.add(1);
            *target = 0x58000051;
            *target.add(1) = 0xd61f0220;
            ptr::copy_nonoverlapping(
                &replace as *const *mut u8 as *const u32,
                target.add(2),
                2,
            );
        } else {
            *original = 0x58000051;
            *original.add(1) = 0xd61f0220;
            ptr::copy_nonoverlapping(
                &replace as *const *mut u8 as *const u32,
                original.add(2),
                2,
            );
        }

        clear_cache(symbol, 5 * 4);
    } else {
        if !trampoline.is_null() {
            fix_instructions(original, 1, trampoline as *mut u32);
        }

        let _code = ProtectedMemory::new(original as *mut u8, 4)?;
        *original = 0x14000000 | ((pc_offset & 0x03ffffff) as u32);
        clear_cache(symbol, 4);
    }

    if !result.is_null() {
        *result = trampoline;
    }

    Ok(if pc_offset.abs() >= (0x03ffffff >> 1) { 5 * 4 } else { 4 })
}
