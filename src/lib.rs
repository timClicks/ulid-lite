use core::fmt::{Display, Formatter, LowerHex, Result, UpperHex};
use libc::{self};
use std::time::SystemTime;

mod base32 {
    use core::hint::unreachable_unchecked;

    const ULID_LEN: usize = 26;

    #[inline]
    fn lookup(b: u8) -> u8 {
        match b {
            0 => b'0',
            1 => b'1',
            2 => b'2',
            3 => b'3',
            4 => b'4',
            5 => b'5',
            6 => b'6',
            7 => b'7',
            8 => b'8',
            9 => b'9',
            10 => b'A',
            11 => b'B',
            12 => b'C',
            13 => b'D',
            14 => b'E',
            15 => b'F',
            16 => b'G',
            17 => b'H',
            18 => b'J',
            19 => b'K',
            20 => b'M',
            21 => b'N',
            22 => b'P',
            23 => b'Q',
            24 => b'R',
            25 => b'S',
            26 => b'T',
            27 => b'V',
            28 => b'W',
            29 => b'X',
            30 => b'Y',
            31 => b'Z',
            // Safety: safe because these bytes are masked out by 0x1f at the call site
            _ => unsafe { unreachable_unchecked() },
        }
    }

    pub fn encode(mut raw_ulid: u128, buffer: &mut [u8]) {
        for i in 0..ULID_LEN {
            buffer[ULID_LEN - 1 - i] = lookup((raw_ulid & 0x1f) as u8);
            raw_ulid >>= 5;
        }
    }
}

#[inline]
fn time_bits() -> u128 {
    // TODO: add OS-specific implementations that are quicker
    let now = SystemTime::now();

    let t = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock is set to before UNIX epoch");

    t.as_millis() & (1 << 48) - 1
}

#[inline]
fn rand_bits() -> u128 {
    let mut bits: u128 = 0;

    // Safety: safe because libc
    let (a, b, c) = unsafe { (libc::rand(), libc::rand(), libc::rand()) };

    bits |= (a as u128) << 64;
    bits |= (b as u128) << 32;
    bits |= c as u128;
    bits &= (1 << 80) - 1; // 0xfff...
    bits
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Ulid {
    bits: u128,
}

impl Ulid {
    #[inline]
    pub fn new() -> Self {
        Ulid {
            bits: time_bits() << 80 | rand_bits(),
        }
    }

    pub fn new_nil() -> Self {
        Ulid { bits: 0 }
    }
}

impl Display for Ulid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf: [u8; 26] = [0; 26];
        base32::encode(self.bits, &mut buf);
        // Safety: guaranteed to be UTF-8 because we control the
        //         bytes that are present.
        let ascii_ulid = unsafe { core::str::from_utf8_unchecked(&buf) };
        write!(f, "{}", ascii_ulid)
    }
}

impl LowerHex for Ulid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerHex::fmt(&self.bits, f)
    }
}

impl UpperHex for Ulid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        UpperHex::fmt(&self.bits, f)
    }
}

pub fn ulid() -> String {
    Ulid::new().to_string()
}

pub fn ulid_raw() -> u128 {
    Ulid::new().bits
}

#[cfg(test)]
mod that {
    use super::*;

    #[test]
    fn each_ulid_is_unique() {
        use itertools::Itertools;

        let ulids: Vec<_> = (0..100).map(|_| ulid()).collect();
        for pair in ulids.into_iter().permutations(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn newer_ulids_are_higher() {
        use std::thread::sleep;
        use std::time::Duration;

        let a = ulid();
        sleep(Duration::from_millis(2));
        let b = ulid();
        assert!(a < b);
    }
}
