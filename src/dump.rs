use std::ffi::c_void;

use crate::{lua_State, lua_dump, LError};

// TODO: Make better writer error handling.
pub unsafe fn dump<WRITER>(
    state: lua_State,
    buffer_writer: &mut WRITER,
) -> std::result::Result<(), LError>
where
    WRITER: std::io::Write,
{
    unsafe extern "C" fn writer_callback<WRITER>(
        _: lua_State,
        data: *const u8,
        size: usize,
        userdata: *mut c_void,
    ) -> i32
    where
        WRITER: std::io::Write,
    {
        let writer = &mut *userdata.cast::<WRITER>();
        match writer.write_all(std::slice::from_raw_parts(data, size)) {
            Ok(_) => 0,
            Err(_) => 1,
        }
    }
    match lua_dump(
        state,
        writer_callback::<WRITER>,
        buffer_writer as *mut WRITER as _,
    ) {
        0 => Ok(()),
        any @ _ => Err(LError::DumpError(any)),
    }
}
