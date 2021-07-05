use core::fmt::{Display, Formatter, LowerHex, Result, UpperHex};
use std::time::{SystemTime, Duration};

use xorshift::{Rand, Rng, SeedableRng, SplitMix64, Xoroshiro128};

#[cfg(miri)]
use libc_shim as libc;

/// Number of bytes for the binary representation of a `ulid`
pub const ULID_BINARY_LEN: usize = 16;

/// Number of bytes for the ASCII text representation of a `ulid`
pub const ULID_LEN: usize = 26;

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
fn duration_since_epoch() -> Duration {
    let now = SystemTime::now();

    now
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock is set to before UNIX epoch")
}

#[repr(C)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Ulid {
    bits: u128,
}

impl Ulid {
    #[inline]
    pub fn new() -> Self {
        UlidGenerator::new().ulid()
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


/// Generates ULIDs, sortable yet unique identifiers.
///
/// The primary way to create a `UlidGenerator` is through `::new()`.
/// This will seed the internal pseudo-random number generator (PRNG)
/// with the current timestamp.
///
/// `UlidGenerator` implements [`std::iter::Iterator`], which allows you
///  to create a continuous series of ULIDs.
///
/// They
///
/// ```rust
/// use ulid::UlidGenerator;
///
/// let mut ulid_gen = UlidGenerator::new();
/// let mut ulids: Vec<_> = ulid_gen.take(1000).collect();
///
/// let test = ulids.pop().unwrap();
/// for ulid in ulids {
///     assert_ne!(test, ulid);
/// }
/// ```
///
/// You can also use a fixed seed to create a repeatable sequence:
///
/// ```rust
/// use ulid::UlidGenerator;
///
/// let ulid_gen = UlidGenerator::from_seed(12345);
/// let ulids: Vec<_> = ulid_gen.take(5).collect();
///
/// // Only the low bits are affected, so we check slices near the end
/// assert_eq!(&(ulids[0].to_string()[20..]), "RBPBCT");
/// assert_eq!(&(ulids[4].to_string()[20..]), "BZBF00");
/// ```
pub struct UlidGenerator {
    rng: Xoroshiro128,
}

impl UlidGenerator {

    #[inline]
    pub fn new() -> Self {
        let seed = (duration_since_epoch().as_nanos() & u64::MAX as u128) as u64;
        Self::from_seed(seed)
    }

    #[inline]
    pub fn from_seed(seed: u64) -> Self {
        // Use a SplitMix64 PRNG to seed a Xoroshiro128+ PRNG
        let mut sm: SplitMix64 = SeedableRng::from_seed(seed);
        let rng: Xoroshiro128 = Rand::rand(&mut sm);

        UlidGenerator {
            rng,
        }
    }

    #[inline]
    pub fn ulid(&mut self) -> Ulid {
        Ulid {
            bits: self.time_bits() << 80 | self.rand_bits()
        }
    }

    #[inline]
    fn time_bits(&self) -> u128 {
        // TODO: add OS-specific implementations that are quicker

        let t = duration_since_epoch();
        t.as_millis() & (1 << 48) - 1
    }

    #[inline]
    fn rand_bits(&mut self) -> u128 {
        let a = self.rng.next_u64() as u128;
        let b = self.rng.next_u64() as u128;

        let mut bits  = a << 64 | b;
        bits &= (1 << 80) - 1; // 0xfff...
        bits
    }
}

impl Iterator for UlidGenerator {
    type Item = Ulid;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.ulid())
    }
}

/// Create a unique ULID as a base32-encoded string
///
/// # Examples
///
/// ```rust
/// use ulid::ulid;
///
/// let a = ulid();
/// let b = ulid();
/// assert_ne!(a, b);
/// ```
pub fn ulid() -> String {
    UlidGenerator::new().ulid().to_string()
}

pub fn ulid_raw() -> u128 {
    UlidGenerator::new().ulid().bits
}

#[cfg(ffi)]
mod ffi {
    use super::*;
    use ::libc::{c_char, c_int, size_t, ERANGE};
    use std::slice::from_raw_parts_mut;

    #[allow(non_camel_case_types)]
    pub type ulid = [u8; ULID_BINARY_LEN];

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
        pub(crate) gen: UlidGenerator,
    }

    impl ulid_ctx {
        #[inline]
        #[must_use]
        unsafe fn ensure_init(ctx: *mut ulid_ctx) -> *mut ulid_ctx {
            if ctx.is_null() {
                ulid_init(0)
            } else {
                ctx
            }
        }
    }

    /// Destroy the `ulid_ctx` object
    ///
    /// # Safety
    ///
    /// Must not be called on the same value twice. This results
    /// in a double free.
    #[no_mangle]
    pub unsafe extern "C" fn ulid_ctx_destroy(ctx: *mut ulid_ctx) {
        Box::from_raw(ctx); // immediately drop
    }

    /// Generate a `ulid_ctx` and seed the random number generator (RNG)
    /// provided by your system's libc implementation of the rand() family.
    ///
    /// Passing 0 as `seed` will seed the random number generator from the
    /// system's clock.
    #[no_mangle]
    pub extern "C" fn ulid_init(seed: u32) -> *mut ulid_ctx {
        let gen = match seed {
            0 => super::UlidGenerator::new(),
            s => {
                super::UlidGenerator::from_seed(s as u64)
            }
        };

        let ctx = ulid_ctx { gen };
        Box::leak(Box::new(ctx))
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
        let ctx = ulid_ctx::ensure_init(ctx);

        let id: ulid = (*ctx).gen.ulid().into();
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

        let ctx = ulid_ctx::ensure_init(ctx);

        let id = (*ctx).gen.ulid();
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
        println!("{:?}", ulids);
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

    #[cfg(ffi)]
    mod ffi {
        use std::{ffi::CStr, os::raw::c_char};

        use crate::*;

        #[test]
        fn can_init_ctx() {
            let ctx = ffi::ulid_init(42);
            let as_u32: u32 = unsafe { (*ctx).gen.rng.gen_range(10, 20) };
            assert!(as_u32 >= 10);
            assert!(as_u32 <= 20);

            let ctx = ffi::ulid_init(0);
            let as_u32: u32 = unsafe { (*ctx).gen.rng.gen_range(10, 20) };
            assert!(as_u32 >= 10);
            assert!(as_u32 <= 20);
        }

        #[test]
        fn can_create_new_ulid() {
            let mut dest = [0u8; ULID_BINARY_LEN];

            unsafe { ffi::ulid_new(std::ptr::null_mut(), &mut dest) };
            assert_ne!(dest, [0u8; ULID_BINARY_LEN]); // should be impossible after 1-1-1970
        }

        #[test]
        fn can_create_new_ulid_as_base32() {
            let mut dest = [0_i8; ULID_LEN + 1];
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;
            let null_ptr = std::ptr::null_mut();

            let ret = unsafe { ffi::ulid_write_new(null_ptr, dest_ptr, dest.len()) };
            assert_eq!(ret, 26);

            // let reconst = unsafe { CStr::from_ptr(dest_ptr) }.to_str().unwrap();
            // assert_eq!(reconst.len(), 26);
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
            let mut id = [0u8; ULID_BINARY_LEN];
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
            let mut id = [0u8; ULID_BINARY_LEN];
            unsafe { ffi::ulid_new(std::ptr::null_mut(), &mut id) };

            let mut dest = [0u8; ULID_LEN]; // one byte too small
            let dest_ptr = dest.as_mut_ptr() as *mut c_char;

            let ret = unsafe { ffi::ulid_write(&id, dest_ptr, dest.len()) };
            assert_eq!(ret, -libc::ERANGE);
            assert_eq!(dest[0], 0); // nothing written to dest
        }
    }
}

