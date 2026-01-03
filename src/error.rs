use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubstrateError {
    #[error("Memory protection failed: {0}")]
    MemoryProtection(String),

    #[error("Memory mapping failed: {0}")]
    MemoryMap(String),

    #[error("Invalid symbol address")]
    InvalidSymbol,

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Library not found: {0}")]
    LibraryNotFound(String),

    #[error("ELF parsing error: {0}")]
    ElfParsing(String),

    #[error("Invalid instruction at {0:#x}")]
    InvalidInstruction(usize),

    #[error("Instruction disassembly failed")]
    DisassemblyFailed,

    #[error("Hook installation failed: {0}")]
    HookFailed(String),

    #[error("Insufficient space for hook")]
    InsufficientSpace,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Null pointer encountered")]
    NullPointer,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, SubstrateError>;
