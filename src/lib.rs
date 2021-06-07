use core::fmt::{Display, Formatter, LowerHex, Result, UpperHex};
use std::time::SystemTime;

#[cfg(not(miri))]
use libc;
#[cfg(miri)]
use libc_shim as libc;

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
pub fn init() -> u32 {
    const SAFE_BITS: i64 = u32::MAX as i64;
    // Safety: safe because no memory is being passed to libc
    unsafe {
        let now = libc::time(0 as *mut _) & SAFE_BITS;
        seed(now as u32);
        now as u32
    }
}

pub fn ulid() -> String {
    Ulid::new().to_string()
}

pub fn ulid_raw() -> u128 {
    Ulid::new().bits
}

//#[cfg(ffi)]
mod ffi {
    use super::*;
    use ::libc::{c_char, c_int, size_t, ERANGE};
    use std::slice::from_raw_parts_mut;

    #[allow(non_camel_case_types)]
    pub type ulid = [u8; 16];

    impl From<Ulid> for ulid {
        #[inline]
        fn from(id: Ulid) -> Self {
            unsafe { std::mem::transmute(id.bits) }
        }
    }

    /// Context object for `ulid` operations
    ///
    /// Contains information related to the internal RNG.
    // #[repr(rust)] so that cbindgen generates an opaque struct
    #[allow(non_camel_case_types)]
    pub struct ulid_ctx {
        pub(crate) seed: u32,
    }

    impl ulid_ctx {
        #[inline]
        unsafe fn ensure_init(ctx: *mut ulid_ctx) {
            if ctx.is_null() {
                ulid_init(0);
            } else if (*ctx).seed == 0 {
                (*ctx).seed = (*ulid_init(0)).seed;
            }
        }
    }

    /// Generate a `ulid_ctx` and seed the random number generator (RNG)
    /// provided by your system's libc implementation of the rand() family.
    ///
    /// Passing 0 as `seed` will seed the random number generator from the
    /// system's clock.
    #[no_mangle]
    pub extern "C" fn ulid_init(seed: u32) -> *mut ulid_ctx {
        let s = match seed {
            0 => init(),
            s => {
                super::seed(s);
                s
            }
        };

        &mut ulid_ctx { seed: s }
    }

    // /// Seed the random number generator with `s`
    // #[no_mangle]
    // pub unsafe extern "C" fn ulid_seed(s: u32) {
    //     seed(s);
    // }

    /// Create a new 128-bit ULID in `dest`.
    ///
    /// If the `ctx` pointer is null, the random number generator is re-seeded
    /// from the system's clock.
    ///
    /// The destination `dest` must be a valid, non-null, pointer to `ulid`.
    #[no_mangle]
    pub unsafe extern "C" fn ulid_new(ctx: *mut ulid_ctx, dest: &mut ulid) {
        ulid_ctx::ensure_init(ctx);

        let id: ulid = Ulid::new().into();
        *dest = std::mem::transmute(id);
    }

    /// Write a new ULID to `dest` as a string.
    ///
    /// Crockford's Base32 alphabet is used, and exactly 27 bytes are written,
    /// including the terminating null byte.
    ///
    /// The destination `dest` must be a valid, non-null, pointer to a `char`
    /// buffer with `size` bytes, and should have at least 27 bytes.
    ///
    /// If the `ctx` pointer is null, the random number generator is re-seeded
    /// from the system's clock.
    ///
    /// Returns the number of characters printed (excluding the terminating null
    /// byte) on success, or a negative error code on failure.
    #[no_mangle]
    pub unsafe extern "C" fn ulid_write_new(
        ctx: *mut ulid_ctx,
        dest: *mut c_char,
        size: size_t,
    ) -> c_int {
        if size < ULID_LEN + 1 {
            return -ERANGE;
        }

        ulid_ctx::ensure_init(ctx);

        let id = Ulid::new();
        let slice = from_raw_parts_mut(dest as *mut u8, size);
        base32::encode(id.bits, slice);
        slice[ULID_LEN] = 0;

        ULID_LEN as c_int // cast is safe because ULID_LEN is tiny
    }

    /// Write the 128-bit ULID pointed by `id` to `dest` as a string.
    ///
    /// Crockford's Base32 alphabet is used, and exactly 27 bytes are written,
    /// including the terminating null byte.
    ///
    /// The destination `dest` must be a valid, non-null, pointer to a `char`
    /// buffer with `size` bytes, and should have at least 27 bytes.
    ///
    /// Returns the number of characters printed (excluding the terminating null
    /// byte) on success, or a negative error code on failure.
    #[no_mangle]
    pub unsafe extern "C" fn ulid_write(id: &ulid, dest: *mut c_char, size: size_t) -> c_int {
        if size < ULID_LEN + 1 {
            return -ERANGE;
        }

        let slice = from_raw_parts_mut(dest as *mut u8, size);
        base32::encode(std::mem::transmute(*id), slice);
        slice[ULID_LEN] = 0;

        ULID_LEN as c_int // cast is safe because ULID_LEN is tiny
    }
}

#[cfg(test)]
mod that {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // too slow and unlikely to pass in Miri
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

    //#[cfg(ffi)]
    mod ffi {
        use std::{ffi::CStr, os::raw::c_char};

        use crate::*;

        #[test]
        fn can_init_ctx() {
            let ctx = ffi::ulid_init(42);
            let as_u32: u32 = unsafe { std::mem::transmute(ctx.as_ref().unwrap().seed) };
            assert_eq!(as_u32, 42);

            let ctx = ffi::ulid_init(0);
            let as_u32: u32 = unsafe { std::mem::transmute(ctx.as_ref().unwrap().seed) };
            assert_ne!(as_u32, 0);
        }

        #[test]
        fn can_create_new_ulid() {
            let mut dest = [0u8; 16];

            unsafe { ffi::ulid_new(std::ptr::null_mut(), &mut dest) };
            assert_ne!(dest, [0u8; 16]); // should be impossible after 1-1-1970
        }

        #[test]
        fn can_create_new_ulid_as_base32() {
            let mut dest = [0u8; 64];
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;
            let null_ptr = std::ptr::null_mut();

            let ret = unsafe { ffi::ulid_write_new(null_ptr, dest_ptr, dest.len()) };
            assert_eq!(ret, 26);

            let reconst = unsafe { CStr::from_ptr(dest_ptr) }.to_str().unwrap();
            assert_eq!(reconst.len(), 26);
        }

        #[test]
        fn creating_new_ulid_as_base32_doesnt_overflow() {
            let mut dest = [0u8; ULID_LEN]; // one byte too small
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;
            let null_ptr = std::ptr::null_mut();

            let ret = unsafe { ffi::ulid_write_new(null_ptr, dest_ptr, dest.len()) };
            assert_eq!(ret, -libc::ERANGE);
            assert_eq!(dest[0], 0); // nothing written to dest
        }

        #[test]
        fn can_encode_binary_ulid_as_base32() {
            let mut id = [0u8; 16];
            unsafe { ffi::ulid_new(std::ptr::null_mut(), &mut id) };

            let mut dest = [0u8; 64];
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;

            let ret = unsafe { ffi::ulid_write(&id, dest_ptr, dest.len()) };
            assert_eq!(ret, 26);

            let reconst = unsafe { CStr::from_ptr(dest_ptr) }.to_str().unwrap();
            assert_eq!(reconst.len(), 26);
        }

        #[test]
        fn encoding_binary_ulid_as_base32_doesnt_overflow() {
            let mut id = [0u8; 16];
            unsafe { ffi::ulid_new(std::ptr::null_mut(), &mut id) };

            let mut dest = [0u8; ULID_LEN]; // one byte too small
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;

            let ret = unsafe { ffi::ulid_write(&id, dest_ptr, dest.len()) };
            assert_eq!(ret, -libc::ERANGE);
            assert_eq!(dest[0], 0); // nothing written to dest
        }
    }
}

/// Mock some libc functions for the tests to run on Miri
///
/// Note: MIRIFLAGS="-Zmiri-disable-isolation" is needed for `SystemTime::now()`.
#[cfg(miri)]
mod libc_shim {
    pub use libc::{c_int, c_uint, size_t, time_t, ERANGE};

    pub unsafe fn rand() -> c_int {
        42
    }

    pub unsafe fn srand(_: c_uint) {}

    pub unsafe fn time(_: *mut time_t) -> time_t {
        42
    }
}
