pub mod raw;

use std::{
    alloc::{alloc, dealloc, Layout},
    os::raw::c_void,
    ptr,
};

const ALIGNMENT: usize = 16;

pub struct MngError {
    pub code: i32,
}

pub struct MngFile {
    handle: Option<raw::MngHandle>,
}

impl MngFile {
    extern "C" fn mem_alloc(len: usize) -> *mut c_void {
        let layout = match Layout::from_size_align(len, ALIGNMENT) {
            Ok(r) => r,
            Err(_) => return ptr::null_mut(),
        };

        unsafe {
            let p = alloc(layout);

            if p.is_null() {
                ptr::null_mut()
            } else {
                // We have to zero the memory as libmng does not properly initialize its structures
                p.write_bytes(0, len);
                p as *mut c_void
            }
        }
    }

    extern "C" fn mem_dealloc(p: *mut c_void, len: usize) {
        let layout = match Layout::from_size_align(len, ALIGNMENT) {
            Ok(r) => r,
            Err(_) => return,
        };

        unsafe {
            dealloc(p as *mut u8, layout);
        }
    }

    fn init(&mut self) -> Result<(), MngError> {
        unsafe {
            self.handle = Some(raw::mng_initialize(
                ptr::null(),
                MngFile::mem_alloc,
                MngFile::mem_dealloc,
                ptr::null(),
            ));
        }

        Ok(())
    }

    fn cleanup(&mut self) {
        unsafe {
            if let Some(ref mut h) = self.handle {
                raw::mng_cleanup(h);
                self.handle = None;
            }
        }
    }

    pub fn open() -> Result<MngFile, MngError> {
        let mut file = MngFile { handle: None };

        file.init()?;

        Ok(file)
    }
}

impl Drop for MngFile {
    fn drop(&mut self) {
        self.cleanup();
    }
}
