use crate::error::{Result, SubstrateError};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const SHT_SYMTAB: u32 = 2;
const SHT_DYNSYM: u32 = 11;
const SHT_STRTAB: u32 = 3;
const STT_FUNC: u8 = 2;

#[repr(C)]
struct Elf32Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf32Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u32,
    sh_addr: u32,
    sh_offset: u32,
    sh_size: u32,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u32,
    sh_entsize: u32,
}

#[repr(C)]
pub struct Elf32Sym {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
}

pub struct SymbolTable {
    pub symbols: Vec<Elf32Sym>,
    pub strings: Vec<u8>,
}

pub struct ElfSymbols {
    pub static_symbols: Option<SymbolTable>,
    pub dynamic_symbols: Option<SymbolTable>,
}

fn pread(file: &mut File, buf: &mut [u8], offset: u64) -> Result<usize> {
    file.seek(SeekFrom::Start(offset))?;
    Ok(file.read(buf)?)
}

unsafe fn read_struct<T>(file: &mut File, offset: u64) -> Result<T> {
    let mut data = vec![0u8; std::mem::size_of::<T>()];
    pread(file, &mut data, offset)?;
    Ok(std::ptr::read(data.as_ptr() as *const T))
}

pub fn load_elf_symbols(filename: &str) -> Result<ElfSymbols> {
    let mut file = File::open(filename)?;

    let ehdr: Elf32Ehdr = unsafe { read_struct(&mut file, 0)? };

    if &ehdr.e_ident[0..4] != &ELF_MAGIC {
        return Err(SubstrateError::ElfParsing("Not an ELF file".to_string()));
    }

    if ehdr.e_shentsize as usize != std::mem::size_of::<Elf32Shdr>() {
        return Err(SubstrateError::ElfParsing("Invalid section header size".to_string()));
    }

    let mut section_headers = Vec::new();
    for i in 0..ehdr.e_shnum {
        let shdr: Elf32Shdr = unsafe {
            read_struct(
                &mut file,
                ehdr.e_shoff as u64 + (i as u64 * ehdr.e_shentsize as u64),
            )?
        };
        section_headers.push(shdr);
    }

    let mut shstrtab = vec![0u8; section_headers[ehdr.e_shstrndx as usize].sh_size as usize];
    pread(
        &mut file,
        &mut shstrtab,
        section_headers[ehdr.e_shstrndx as usize].sh_offset as u64,
    )?;

    let mut symtab_hdr: Option<&Elf32Shdr> = None;
    let mut strtab_hdr: Option<&Elf32Shdr> = None;
    let mut dynsym_hdr: Option<&Elf32Shdr> = None;
    let mut dynstr_hdr: Option<&Elf32Shdr> = None;

    for shdr in &section_headers {
        match shdr.sh_type {
            SHT_SYMTAB => {
                if symtab_hdr.is_some() {
                    return Err(SubstrateError::ElfParsing("Multiple symbol tables".to_string()));
                }
                symtab_hdr = Some(shdr);
            }
            SHT_DYNSYM => {
                if dynsym_hdr.is_some() {
                    return Err(SubstrateError::ElfParsing("Multiple dynamic symbol tables".to_string()));
                }
                dynsym_hdr = Some(shdr);
            }
            SHT_STRTAB => {
                let name_offset = shdr.sh_name as usize;
                if name_offset < shstrtab.len() {
                    let name = std::ffi::CStr::from_bytes_until_nul(&shstrtab[name_offset..])
                        .ok()
                        .and_then(|s| s.to_str().ok());
                    if let Some(n) = name {
                        if n == ".strtab" {
                            strtab_hdr = Some(shdr);
                        } else if n == ".dynstr" {
                            dynstr_hdr = Some(shdr);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let static_symbols = if let (Some(symh), Some(strh)) = (symtab_hdr, strtab_hdr) {
        Some(load_symbol_table(&mut file, symh, strh)?)
    } else {
        None
    };

    let dynamic_symbols = if let (Some(dynh), Some(dstrh)) = (dynsym_hdr, dynstr_hdr) {
        Some(load_symbol_table(&mut file, dynh, dstrh)?)
    } else {
        None
    };

    Ok(ElfSymbols {
        static_symbols,
        dynamic_symbols,
    })
}

fn load_symbol_table(
    file: &mut File,
    symh: &Elf32Shdr,
    strh: &Elf32Shdr,
) -> Result<SymbolTable> {
    if symh.sh_size % std::mem::size_of::<Elf32Sym>() as u32 != 0 {
        return Err(SubstrateError::ElfParsing("Invalid symbol table size".to_string()));
    }

    let num_syms = symh.sh_size as usize / std::mem::size_of::<Elf32Sym>();
    let mut symbols = Vec::with_capacity(num_syms);

    for i in 0..num_syms {
        let sym: Elf32Sym = unsafe {
            read_struct(
                file,
                symh.sh_offset as u64 + (i * std::mem::size_of::<Elf32Sym>()) as u64,
            )?
        };
        symbols.push(sym);
    }

    let mut strings = vec![0u8; strh.sh_size as usize];
    pread(file, &mut strings, strh.sh_offset as u64)?;

    Ok(SymbolTable { symbols, strings })
}

pub fn lookup_symbol(symbols: &ElfSymbols, name: &str) -> Option<usize> {
    if let Some(ref dyn_syms) = symbols.dynamic_symbols {
        if let Some(addr) = lookup_in_table(dyn_syms, name) {
            return Some(addr);
        }
    }

    if let Some(ref static_syms) = symbols.static_symbols {
        if let Some(addr) = lookup_in_table(static_syms, name) {
            return Some(addr);
        }
    }

    None
}

fn lookup_in_table(table: &SymbolTable, name: &str) -> Option<usize> {
    for sym in &table.symbols {
        let st_type = sym.st_info & 0xf;
        if st_type != STT_FUNC {
            continue;
        }

        let name_offset = sym.st_name as usize;
        if name_offset < table.strings.len() {
            if let Ok(sym_name) = std::ffi::CStr::from_bytes_until_nul(&table.strings[name_offset..])
            {
                if let Ok(sym_str) = sym_name.to_str() {
                    if sym_str == name {
                        return Some(sym.st_value as usize);
                    }
                }
            }
        }
    }

    None
}
