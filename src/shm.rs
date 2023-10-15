use std::{
    ffi::CString,
    os::fd::{FromRawFd, OwnedFd, RawFd},
};

use libc::{self};
use rand::{self, RngCore};

fn create_shm_file() -> Result<RawFd, &'static str> {
    let mut rng = rand::thread_rng();

    let name = rng.next_u64().to_string();
    let name = CString::new(name).unwrap();
    let name_ptr = name.into_raw();

    unsafe {
        let fd = libc::shm_open(name_ptr, libc::O_RDWR | libc::O_CREAT | libc::O_EXCL, 0600);

        // retake ptr to free memory
        let _ = CString::from_raw(name_ptr);

        // allocaiton success
        if fd >= 0 {
            return Ok(fd);
        }
    }

    return Err("Unable to allocate shared memory region");
}

pub fn allocate_shm_file(size: i32) -> Result<OwnedFd, &'static str> {
    let fd = create_shm_file()?;

    unsafe {
        let ret = libc::ftruncate(fd, size.into());

        if ret < 0 {
            libc::close(fd);
            return Err("Unable to resize shared memory region");
        } else {
            return Ok(OwnedFd::from_raw_fd(fd));
        }
    }
}
