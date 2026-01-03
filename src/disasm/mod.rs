pub mod hde64;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod arm_decoder;
