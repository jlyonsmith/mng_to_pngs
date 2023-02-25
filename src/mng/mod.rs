pub mod raw;

use self::raw::{MngBool, MngHandle};
use libz_sys;
use std::{
    alloc::{alloc, dealloc, Layout},
    ffi::CString,
    fmt,
    mem::transmute,
    os::raw::{c_uchar, c_uint, c_ulong, c_void},
    path::Path,
    ptr,
};

pub fn null_crc32() -> u32 {
    unsafe { libz_sys::crc32(0, ptr::null(), 0) as u32 }
}
pub fn crc32(crc: u32, buf: &[u8]) -> u32 {
    unsafe { libz_sys::crc32(crc as c_ulong, buf.as_ptr(), buf.len() as u32) as u32 }
}

const ALIGNMENT: usize = 16;

#[derive(Clone, Debug, PartialEq)]
pub struct MngError {
    pub code: i32,
}

impl MngError {
    pub fn new(code: i32) -> MngError {
        MngError { code }
    }
}

impl fmt::Display for MngError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "code {}", self.code)
    }
}

impl std::error::Error for MngError {}

#[derive(Debug)]
pub enum Chunk {
    IHdr {
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        compression: u8,
        filter: u8,
        interlace: u8,
    },
    IDat {
        data: Vec<u8>,
    },
    IEnd,
}

pub struct MngFile {}

#[derive(Debug)]
struct MngFileData<'a> {
    mng_fd: i32,
    chunks: &'a mut Vec<Chunk>,
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

    extern "C" fn open_stream(_: MngHandle) -> MngBool {
        // We open the input file elsewhere
        return 1;
    }

    extern "C" fn close_stream(_: MngHandle) -> MngBool {
        // We close the input file elsewhere
        return 1;
    }

    extern "C" fn read_data(
        handle: MngHandle,
        buf: *mut c_void,
        bytes_to_read: c_uint,
        bytes_read: *mut c_uint,
    ) -> MngBool {
        unsafe {
            let user_data =
                transmute::<*mut c_void, &mut MngFileData>(raw::mng_get_userdata(handle));

            let n = libc::read(user_data.mng_fd, buf, bytes_to_read.try_into().unwrap());

            if n <= 0 {
                return 0;
            }

            *bytes_read = n as c_uint;
        }

        return 1;
    }

    extern "C" fn iterate_chunks(
        handle: MngHandle,
        chunk: MngHandle,
        chunk_type: c_uint,
        _: c_uint,
    ) -> MngBool {
        unsafe {
            let user_data =
                transmute::<*mut c_void, &mut MngFileData>(raw::mng_get_userdata(handle));

            match chunk_type {
                raw::MNG_UINT_IHDR => {
                    let mut width: c_uint = 0;
                    let mut height: c_uint = 0;
                    let mut bit_depth: c_uchar = 0;
                    let mut color_type: c_uchar = 0;
                    let mut compression: c_uchar = 0;
                    let mut filter: c_uchar = 0;
                    let mut interlace: c_uchar = 0;

                    raw::mng_getchunk_ihdr(
                        handle,
                        chunk,
                        &mut width,
                        &mut height,
                        &mut bit_depth,
                        &mut color_type,
                        &mut compression,
                        &mut filter,
                        &mut interlace,
                    );

                    user_data.chunks.push(Chunk::IHdr {
                        width,
                        height,
                        bit_depth,
                        color_type,
                        compression,
                        filter,
                        interlace,
                    })
                }
                raw::MNG_UINT_IDAT => {
                    let mut raw_len: c_uint = 0;
                    let mut raw_data: *mut c_uchar = ptr::null_mut();

                    raw::mng_getchunk_idat(handle, chunk, &mut raw_len, &mut raw_data);

                    let mut new_data = Vec::with_capacity(raw_len as usize);

                    ptr::copy_nonoverlapping(raw_data, new_data.as_mut_ptr(), raw_len as usize);

                    new_data.set_len(raw_len as usize);
                    user_data.chunks.push(Chunk::IDat { data: new_data });
                }
                raw::MNG_UINT_IEND => {
                    user_data.chunks.push(Chunk::IEnd);
                }
                _ => return 1,
            }
        }
        return 1;
    }

    fn check(ret_code: raw::MngRetCode) -> Result<(), MngError> {
        if ret_code != 0 {
            Err(MngError::new(ret_code as i32))
        } else {
            Ok(())
        }
    }

    pub fn get_chunks<P: AsRef<Path>>(path: P, chunks: &mut Vec<Chunk>) -> Result<(), MngError> {
        let mut data = Box::new(MngFileData { mng_fd: 0, chunks });

        unsafe {
            let user_data = transmute::<&mut MngFileData, *mut c_void>(&mut data);

            let mut handle = raw::mng_initialize(
                user_data,
                MngFile::mem_alloc,
                MngFile::mem_dealloc,
                ptr::null(),
            );

            if handle.is_null() {
                return Err(MngError { code: 0 });
            }

            MngFile::check(raw::mng_setcb_openstream(handle, MngFile::open_stream))?;
            MngFile::check(raw::mng_setcb_closestream(handle, MngFile::close_stream))?;
            MngFile::check(raw::mng_setcb_readdata(handle, MngFile::read_data))?;

            let file_name = CString::new(path.as_ref().to_string_lossy().as_bytes()).unwrap();

            let fd = libc::open(file_name.as_ptr(), libc::O_RDONLY);

            if fd < 0 {
                return Err(MngError::new(-1));
            }

            data.mng_fd = fd;

            MngFile::check(raw::mng_read(handle))?;
            MngFile::check(raw::mng_iterate_chunks(handle, 0, MngFile::iterate_chunks))?;

            if data.mng_fd != 0 {
                libc::close(data.mng_fd);
            }

            raw::mng_cleanup(&mut handle);
        }

        Ok(())
    }
}
