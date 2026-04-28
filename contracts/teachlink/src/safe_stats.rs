//! Small helpers for checked arithmetic on cumulative statistics.
/// Safely increment a `u64`. Returns (new_value, overflowed)
pub fn safe_inc_u64(v: u64) -> (u64, bool) {
    match v.checked_add(1) {
        Some(n) => (n, false),
        None => (0u64, true),
    }
}

/// Safely increment a `u32`. Returns (new_value, overflowed)
pub fn safe_inc_u32(v: u32) -> (u32, bool) {
    match v.checked_add(1) {
        Some(n) => (n, false),
        None => (0u32, true),
    }
}

/// Safely add two `i128` values. Returns (new_value, overflowed)
pub fn safe_add_i128(a: i128, b: i128) -> (i128, bool) {
    match a.checked_add(b) {
        Some(n) => (n, false),
        None => (0i128, true),
    }
}
