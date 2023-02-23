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
            Err(e) => return ptr::null_mut(),
        };

        unsafe {
            let p = alloc(layout);

            if p.is_null() {
                ptr::null_mut()
            } else {
                p as *mut c_void
            }
        }
    }

    extern "C" fn mem_dealloc(ptr: *mut c_void, len: usize) {
        let layout = match Layout::from_size_align(len, ALIGNMENT) {
            Ok(r) => r,
            Err(e) => return,
        };

        unsafe {
            dealloc(ptr as *mut u8, layout);
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
            if let Some(h) = self.handle {
                raw::mng_cleanup(h);
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
