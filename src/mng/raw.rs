use std::os::raw::{c_int, c_schar, c_uchar, c_uint, c_void};

pub type MngHandle = *mut c_void;
pub type MngRetCode = c_int;
pub type MngBool = c_schar; // WTF?!?

// The `libmng` C callbacks types
pub type MngMemMalloc = extern "C" fn(len: usize) -> *mut c_void;
pub type MngMemFree = extern "C" fn(ptr: *mut c_void, len: usize);
pub type MngOpenStream = extern "C" fn(handle: MngHandle) -> MngBool;
pub type MngCloseStream = extern "C" fn(handle: MngHandle) -> MngBool;
pub type MngReadData = extern "C" fn(
    handle: MngHandle,
    buf: *mut c_void,
    buf_byte_size: c_uint,
    bytes_read: *mut c_uint,
) -> MngBool;
pub type MngIterateChunks = extern "C" fn(
    handle: MngHandle,
    chunk: MngHandle,
    chunk_type: c_int,
    chunk_sequence: c_uint,
) -> MngBool;

// The `libmng` exported C functions
#[link(name = "mng")]
extern "C" {
    pub fn mng_initialize(
        user_data: *const c_void,
        mem_alloc_cb: MngMemMalloc,
        mem_free_cb: MngMemFree,
        trace_cb: *const c_void,
    ) -> MngHandle;
    pub fn mng_cleanup(handle: *mut MngHandle) -> MngRetCode;
    pub fn mng_setcb_openstream(handle: MngHandle, cb: MngOpenStream) -> MngRetCode;
    pub fn mng_setcb_closestream(handle: MngHandle, cb: MngCloseStream) -> MngRetCode;
    pub fn mng_setcb_readdata(handle: MngHandle, cb: MngReadData) -> MngRetCode;
    pub fn mng_read(handle: MngHandle) -> MngRetCode;
    pub fn mng_iterate_chunks(
        handle: MngHandle,
        start_chunk: c_uint,
        cb: MngIterateChunks,
    ) -> MngRetCode;
    pub fn mng_getchunk_ihdr(
        handle: MngHandle,
        chunk: MngHandle,
        width: *mut c_uint,
        height: *mut c_uint,
        bit_depth: *mut c_uchar,
        color_type: *mut c_uchar,
        compression: *mut c_uchar,
        filter: *mut c_uchar,
        interlace: *mut c_uchar,
    ) -> MngRetCode;
    pub fn mng_getchunk_idat(
        handle: MngHandle,
        chunk: MngHandle,
        raw_len: *mut c_uint,
        raw_data: *mut c_uchar,
    ) -> MngRetCode;
}

const MNG_UINT_IHDR: c_int = 0x49484452;
const MNG_UINT_IDAT: c_int = 0x49444154;
const MNG_UINT_IEND: c_int = 0x49454e44;
