use std::sync::atomic::{AtomicBool, Ordering};

pub static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_debug() {
    DEBUG_ENABLED.store(true, Ordering::Relaxed);
}

pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

#[cfg(feature = "debug")]
pub fn log_hex(data: &[u8], stride: usize, mark: Option<&str>) {
    if !is_debug_enabled() {
        return;
    }

    const HEX_WIDTH: usize = 16;
    let mut i = 0;

    while i < data.len() {
        if i % HEX_WIDTH == 0 {
            if let Some(m) = mark {
                print!("\n[{}] ", m);
            }
            print!("0x{:03x}:", i);
        }

        print!(" ");
        for q in (0..stride).rev() {
            if i + q < data.len() {
                print!("{:02x}", data[i + stride - q - 1]);
            }
        }

        i += stride;

        if i % HEX_WIDTH == 0 && i <= data.len() {
            print!(" ");
            for j in (i.saturating_sub(HEX_WIDTH))..i.min(data.len()) {
                let c = data[j];
                let ch = if c >= 0x20 && c < 0x80 { c as char } else { '.' };
                print!("{}", ch);
            }
            println!();
        }
    }

    if i % HEX_WIDTH != 0 {
        let remaining = HEX_WIDTH - (i % HEX_WIDTH);
        for _ in 0..remaining {
            print!("   ");
        }
        print!(" ");
        for j in (i / HEX_WIDTH * HEX_WIDTH)..(i.min(data.len())) {
            let c = data[j];
            let ch = if c >= 0x20 && c < 0x80 { c as char } else { '.' };
            print!("{}", ch);
        }
        println!();
    }
}

#[cfg(not(feature = "debug"))]
pub fn log_hex(_data: &[u8], _stride: usize, _mark: Option<&str>) {}
