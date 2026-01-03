use crate::error::{Result, SubstrateError};
use crate::symbol::elf::{load_elf_symbols, lookup_symbol};
use crate::symbol::memmap::load_memory_maps;

pub fn find_symbol_address(pid: libc::pid_t, symbol_name: &str, library_name: &str) -> Result<usize> {
    let maps = load_memory_maps(pid)?;

    let mut library_path: Option<String> = None;
    let mut library_base: Option<usize> = None;

    for map in maps {
        if map.name == "[memory]" {
            continue;
        }

        if let Some(pos) = map.name.rfind('/') {
            let basename = &map.name[pos + 1..];
            if basename.starts_with(library_name) && (basename.ends_with(".so") || basename.contains(".so.")) {
                library_path = Some(map.name.clone());
                library_base = Some(map.start);
                unsafe {
                    libc::mprotect(
                        map.start as *mut libc::c_void,
                        map.end - map.start,
                        libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                    );
                }
                break;
            }
        }
    }

    let library_path = library_path.ok_or_else(|| {
        SubstrateError::LibraryNotFound(library_name.to_string())
    })?;

    let library_base = library_base.ok_or_else(|| {
        SubstrateError::LibraryNotFound(library_name.to_string())
    })?;

    let symbols = load_elf_symbols(&library_path)?;

    let symbol_offset = lookup_symbol(&symbols, symbol_name).ok_or_else(|| {
        SubstrateError::SymbolNotFound(symbol_name.to_string())
    })?;

    Ok(library_base + symbol_offset)
}
