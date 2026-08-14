//! Octet lookup tables and hex helpers for percent-encoding.

const fn gen_octet_table(hi: bool) -> [u8; 256] {
    let mut out = [0xff; 256];
    let shift = if hi { 4 } else { 0 };

    let mut i = 0;
    while i < 10 {
        out[(i + b'0') as usize] = i << shift;
        i += 1;
    }
    while i < 16 {
        out[(i - 10 + b'A') as usize] = i << shift;
        out[(i - 10 + b'a') as usize] = i << shift;
        i += 1;
    }
    out
}

#[cfg(test)]
pub(super) const OCTET_TABLE_HI: &[u8; 256] = &gen_octet_table(true);
pub(super) const OCTET_TABLE_LO: &[u8; 256] = &gen_octet_table(false);

/// Decodes a percent-encoded octet, assuming that the bytes are
/// hexadecimal.
#[cfg(test)]
pub(super) fn decode_octet(hi: u8, lo: u8) -> u8 {
    debug_assert!(hi.is_ascii_hexdigit() && lo.is_ascii_hexdigit());
    OCTET_TABLE_HI[hi as usize] | OCTET_TABLE_LO[lo as usize]
}

pub(crate) fn decode_hexdigit(x: u8) -> Option<u8> {
    Some(OCTET_TABLE_LO[x as usize]).filter(|&v| v < 128)
}

pub(crate) const fn is_hexdig(x: u8) -> bool {
    OCTET_TABLE_LO[x as usize] < 128
}

pub(crate) const fn is_hexdig_pair(hi: u8, lo: u8) -> bool {
    OCTET_TABLE_LO[hi as usize] | OCTET_TABLE_LO[lo as usize] < 128
}
