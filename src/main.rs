use libc;
use ulid_lite::ulid;

fn main() {
    unsafe {
        let now = libc::time(0 as *mut _);
        let now_u32 = (now & u32::MAX as i64) as u32;
        libc::srand(now_u32);
    }

    println!("{}", ulid());
}
