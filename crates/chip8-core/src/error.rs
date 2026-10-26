use thiserror::Error;

/// Everything that can go wrong inside the emulator core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Chip8Error {
    /// The ROM does not fit between the entry point (0x200) and the end of memory.
    #[error("ROM is {size} bytes, but only {max} bytes fit in memory")]
    RomTooLarge { size: usize, max: usize },

    /// The CPU fetched an opcode it does not implement.
    #[error("unknown opcode {0:#06X}")]
    UnknownOpcode(u16),

    /// A call (`2NNN`) was made with all 16 stack slots in use.
    #[error("stack overflow (more than 16 nested calls)")]
    StackOverflow,

    /// A return (`00EE`) was made with an empty stack.
    #[error("stack underflow (return with no call)")]
    StackUnderflow,
}

/// Shorthand so functions can return `Result<T>`.
pub type Result<T> = core::result::Result<T, Chip8Error>;