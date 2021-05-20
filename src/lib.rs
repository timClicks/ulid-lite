use core::fmt::{Display, Formatter, LowerHex, Result, UpperHex};
use libc::{self};
use std::{slice::from_raw_parts_mut, time::SystemTime};
use std::os::raw::c_char;

const ULID_LEN: usize = 26;
mod base32 {
    use super::ULID_LEN;
    use core::hint::unreachable_unchecked;

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

#[repr(C)]
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

/// Sets the seed of the internal random number generator.
///
/// This function is provided so that you can retain
/// full control. Most applications will prefer to
/// call `init()`.
pub fn seed(s: u32) {
    // Safety: safe because no memory is being passed to libc
    unsafe {
        libc::srand(s);
    }
}

/// Initialize the internal random number generator
/// based on the system's clock.
pub fn init() {
    const SAFE_BITS:i64 = u32::MAX as i64;
    // Safety: safe because no memory is being passed to libc
    unsafe {
        let now = libc::time(0 as *mut _) & SAFE_BITS;
        seed(now as u32);
    }
}

pub type UlidArray = [u8; 16];

impl From<Ulid> for UlidArray {
    #[inline]
    fn from(id: Ulid) -> Self {
        unsafe {
            std::mem::transmute(id.bits)
        }
    }
}

pub fn ulid() -> String {
    Ulid::new().to_string()
}

pub fn ulid_raw() -> u128 {
    Ulid::new().bits
}

/// Initialize the random number generator from the system's clock
#[no_mangle]
pub unsafe extern "C" fn ulid_init() {
    init();
}

/// Seed the random number generator with `s`
#[no_mangle]
pub unsafe extern "C" fn ulid_seed(s: u32) {
    seed(s);
}

/// Create a new ULID
///
/// Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
///       has been called before this function.
#[no_mangle]
pub unsafe extern "C" fn ulid_new() -> *mut UlidArray {
    &mut Ulid::new().into() as &mut _
}

/// Create a new ULID and encodes it as a Crockford Base32 string.
///
/// Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
///       has been called before this function.
///
/// Note: This function allocates memory. Callers are required to free
///       the return value when is no longer useful.
#[no_mangle]
pub unsafe extern "C" fn ulid_new_string() -> *mut c_char {
    let ptr = Ulid::new().to_string().as_ptr();
    std::mem::transmute(ptr) // legal because of the base32 alphabet
}

/// Create a new ULID and write it to `buf`. 
///
/// Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
/// has been called before this function.
///
/// Warning: callers must ensure that `buf` is (at least) 26 bytes.
#[no_mangle]
pub unsafe extern "C" fn ulid_write_new(buf: &mut c_char) {
    let id = Ulid::new();
    let slice = from_raw_parts_mut(buf, ULID_LEN);
    base32::encode(id.bits, std::mem::transmute(slice));
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
