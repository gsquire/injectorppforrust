#![cfg(target_arch = "aarch64")]

use crate::injector_core::utils::*;

// C6.2.220 RET
// Return from subroutine branches unconditionally to an address in a register, with a hint that this is a subroutine return.
// x30 is used to hold the address to be branched to.
pub(crate) fn emit_ret_x30() -> [bool; 32] {
    emit_ret(&u8_to_bits::<5>(30))
}

// C6.2.220 RET
// Return from subroutine branches unconditionally to an address in a register,
// with a hint that this is a subroutine return.
pub(crate) fn emit_ret(register_name: &[bool; 5]) -> [bool; 32] {
    let mut code_bits = [false; 32];
    let mut cur = 0;

    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;

    for &bit in register_name.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = false;
    cur += 1;
    code_bits[cur] = true;
    cur += 1;
    code_bits[cur] = true;

    code_bits
}

/// Emit a 32‑bit BR (Branch to Register) instruction from a 5‑bit register name.
///
/// The instruction is built by concatenating fixed bit fields and the provided
/// register bits in the following order:
///
/// 1. 5 bits: 0,0,0,0,0  
/// 2. 5 bits: register_name  
/// 3. 2 bits: 0,0  
/// 4. 4 bits: 0,0,0,0
/// 5. 5 bits: 1,1,1,1,1  
/// 6. 2 bits: 0,0  
/// 7. 1 bit: 0  
/// 8. 1 bit: 0  
/// 9. 5 bits: 1,1,0,1,0  
///
/// Total: 5 + 5 + 2 + 6 + 5 + 2 + 1 + 1 + 5 = 32 bits.
pub(crate) fn emit_br(register_name: [bool; 5]) -> [bool; 32] {
    let mut code_bits = [false; 32];
    let mut cur = 0;

    // Group 1: 5 bits of 0.
    for _ in 0..5 {
        code_bits[cur] = false;
        cur += 1;
    }

    // Group 2: 5 bits from register_name.
    for &bit in register_name.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Group 3: 2 bits of 0.
    code_bits[cur] = false;
    cur += 1;

    code_bits[cur] = false;
    cur += 1;

    // Group 4: 4 bits of 0.
    for _ in 0..4 {
        code_bits[cur] = false;
        cur += 1;
    }

    // Group 5: 5 bits of 1.
    for _ in 0..5 {
        code_bits[cur] = true;
        cur += 1;
    }

    // Group 6: 2 bits of 0.
    for _ in 0..2 {
        code_bits[cur] = false;
        cur += 1;
    }

    // Group 7: 1 bit of 0.
    code_bits[cur] = false;
    cur += 1;

    // Group 8: 1 bit of 0.
    code_bits[cur] = false;
    cur += 1;

    // Group 9 (adjusted): 5 bits: 1, 1, 0, 1, 0, 1, 1
    let group9 = [true, true, false, true, false, true, true];
    for &bit in group9.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    code_bits
}

/// Converts a 64-bit address into a 32-bit instruction encoding.
/// It extracts 16 bits starting at `start` from the 64-bit address,
/// then calls `emit_movk` to build the final 32-bit code.
///
/// # Parameters
/// - `address`: The 64-bit address value.
/// - `start`: The starting bit index from which to extract 16 bits.
/// - `sf`: A flag bit.
/// - `hw`: A 2-bit value, represented as a [bool; 2].
/// - `register_name`: A 5-bit value, represented as a [bool; 5].
///
/// # Returns
/// A 32-bit code represented as a [bool; 32].
pub(crate) fn emit_movk_from_address(
    address: u64,
    start: usize,
    sf: bool,
    hw: [bool; 2],
    register_name: [bool; 5],
) -> [bool; 32] {
    let address_bits = u64_to_bits(address);
    let mut value_bits = [false; 16];
    value_bits.copy_from_slice(&address_bits[start..(16 + start)]);
    emit_movk(value_bits, sf, hw, register_name)
}

/// Builds the 32-bit instruction encoding by concatenating:
/// 1. The 5-bit register name.
/// 2. The 16-bit immediate value (`value_bits`).
/// 3. The 2-bit `hw` value.
/// 4. Fixed bits: 1,0,1,0,0,1 then 1,1.
/// 5. Finally the `sf` bit.
///
/// The total bit-length is 5 + 16 + 2 + 6 + 2 + 1 = 32 bits.
///
/// # Parameters
/// - `value_bits`: A 16-bit immediate value as [bool; 16].
/// - `sf`: A flag bit.
/// - `hw`: A 2-bit value as [bool; 2].
/// - `register_name`: A 5-bit value as [bool; 5].
///
/// # Returns
/// A 32-bit code represented as a [bool; 32].
pub(crate) fn emit_movk(
    value_bits: [bool; 16],
    sf: bool,
    hw: [bool; 2],
    register_name: [bool; 5],
) -> [bool; 32] {
    let mut code_bits = [false; 32];
    let mut cur = 0;

    // Append register_name bits.
    for &bit in register_name.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append immediate (value_bits).
    for &bit in value_bits.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append hw bits.
    for &bit in hw.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append fixed bits: 1, 0, 1, 0, 0, 1.
    let fixed_bits1 = [true, false, true, false, false, true];
    for &bit in fixed_bits1.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append fixed bits: 1, 1.
    let fixed_bits2 = [true, true];
    for &bit in fixed_bits2.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append the sf bit.
    code_bits[cur] = sf;

    code_bits
}

/// Extracts a 16-bit immediate value from `address` starting at bit `start`
/// and then builds the final 32-bit MOVZ instruction.
///
/// # Parameters
/// - `address`: The 64-bit address value.
/// - `start`: The starting bit index for extraction.
/// - `sf`: A flag bit.
/// - `hw`: A 2-bit value as a [bool; 2].
/// - `register_name`: A 5-bit register name as a [bool; 5].
///
/// # Returns
/// A 32-bit code represented as a [bool; 32].
pub(crate) fn emit_movz_from_address(
    address: u64,
    start: usize,
    sf: bool,
    hw: [bool; 2],
    register_name: [bool; 5],
) -> [bool; 32] {
    let address_bits = u64_to_bits(address);
    let mut value_bits = [false; 16];
    value_bits.copy_from_slice(&address_bits[start..(16 + start)]);
    emit_movz(value_bits, sf, hw, register_name)
}

/// Assembles a 32-bit MOVZ instruction by concatenating:
/// 1. The 5-bit register name.
/// 2. The 16-bit immediate value (`value_bits`).
/// 3. The 2-bit hardware field (`hw`).
/// 4. Fixed bits: 1,0,1,0,0,1 followed by 0,1.
/// 5. Finally, the `sf` bit.
///
/// The bit ordering is maintained so that the final instruction is 32 bits long.
///
/// # Parameters
/// - `value_bits`: A 16-bit immediate value as a [bool; 16].
/// - `sf`: A flag bit.
/// - `hw`: A 2-bit value as a [bool; 2].
/// - `register_name`: A 5-bit register name as a [bool; 5].
///
/// # Returns
/// A 32-bit instruction encoded as a [bool; 32].
pub(crate) fn emit_movz(
    value_bits: [bool; 16],
    sf: bool,
    hw: [bool; 2],
    register_name: [bool; 5],
) -> [bool; 32] {
    let mut code_bits = [false; 32];
    let mut cur = 0;

    // Append register_name bits.
    for &bit in register_name.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append immediate (value_bits).
    for &bit in value_bits.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append hw bits.
    for &bit in hw.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append fixed bits: 1, 0, 1, 0, 0, 1.
    let fixed_bits1 = [true, false, true, false, false, true];
    for &bit in fixed_bits1.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append fixed bits: 0, 1.
    let fixed_bits2 = [false, true];
    for &bit in fixed_bits2.iter() {
        code_bits[cur] = bit;
        cur += 1;
    }

    // Append the sf bit.
    code_bits[cur] = sf;

    code_bits
}
