use std::sync::atomic::{AtomicBool, Ordering};

use burt_command::burt_command;

pub static BVAR_TAS_QUANTIZE_MOVEMENT: AtomicBool = AtomicBool::new(true);

/// Whether to quantize movement. Quantization is used
/// to match the heavily restricted movement possibilities
/// in leaked Portal 2 builds caused by lower joystick precision.
///
/// When enabled, movement vectors shorter than `175.0` are quantized to the nearest
/// 16-bit XInput step (`450.0 / 32768.0`). Vectors longer than `175.0` are scaled
/// to exactly `450.0` to let the engine perform precise `f32` normalization.
///
/// Disable this to allow arbitrary `f32` precision.
#[burt_command(var = BVAR_TAS_QUANTIZE_MOVEMENT)]
pub fn bvar_tas_quantize_movement(quantize: bool) {
    BVAR_TAS_QUANTIZE_MOVEMENT.store(quantize, Ordering::Relaxed);
}
