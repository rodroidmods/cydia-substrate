use crate::error::{Result, SubstrateError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct MemoryMap {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub fn load_memory_maps(pid: libc::pid_t) -> Result<Vec<MemoryMap>> {
    let path = format!("/proc/{}/maps", pid);
    let file = File::open(&path).map_err(|e| {
        SubstrateError::Io(e)
    })?;

    let reader = BufReader::new(file);
    let mut maps = Vec::new();
    let mut map_dict: HashMap<String, (usize, usize)> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 1 {
            continue;
        }

        let addr_parts: Vec<&str> = parts[0].split('-').collect();
        if addr_parts.len() != 2 {
            continue;
        }

        let start = usize::from_str_radix(addr_parts[0], 16)
            .map_err(|_| SubstrateError::ElfParsing("Invalid address".to_string()))?;
        let end = usize::from_str_radix(addr_parts[1], 16)
            .map_err(|_| SubstrateError::ElfParsing("Invalid address".to_string()))?;

        let name = if parts.len() >= 6 {
            parts[5].to_string()
        } else {
            "[memory]".to_string()
        };

        if let Some((existing_start, existing_end)) = map_dict.get_mut(&name) {
            if start < *existing_start {
                *existing_start = start;
            }
            if end > *existing_end {
                *existing_end = end;
            }
        } else {
            map_dict.insert(name.clone(), (start, end));
        }
    }

    for (name, (start, end)) in map_dict {
        maps.push(MemoryMap { name, start, end });
    }

    Ok(maps)
}

pub fn find_library_base(pid: libc::pid_t, lib_name: &str) -> Result<usize> {
    let maps = load_memory_maps(pid)?;

    for map in maps {
        if map.name == "[memory]" {
            continue;
        }

        if let Some(pos) = map.name.rfind('/') {
            let basename = &map.name[pos + 1..];
            if basename.starts_with(lib_name) && (basename.len() == lib_name.len() + 3 && basename.ends_with(".so") || basename.len() > lib_name.len() + 3) {
                unsafe {
                    libc::mprotect(
                        map.start as *mut libc::c_void,
                        map.end - map.start,
                        libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                    );
                }
                return Ok(map.start);
            }
        }
    }

    Err(SubstrateError::LibraryNotFound(lib_name.to_string()))
}
