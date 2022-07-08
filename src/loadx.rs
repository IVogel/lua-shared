use std::{ffi::c_void, hint::unreachable_unchecked};

use crate::{lua_State, lua_loadx, pop, tolstring, LError, Status};

pub unsafe fn loadx<READER>(
    state: lua_State,
    reader: &mut READER,
    chunk_name: *const u8,
    mode: *const u8,
) -> std::result::Result<(), LError>
where
    READER: std::io::Read,
{
    struct ReaderState<'a, READER>
    where
        READER: std::io::Read,
    {
        buffer: [u8; 4096],
        reader: &'a mut READER,
    }
    unsafe extern "C" fn reader_callback<READER>(
        _: lua_State,
        userdata: *mut c_void,
        size: &mut usize,
    ) -> *const u8
    where
        READER: std::io::Read,
    {
        let reader = &mut *userdata.cast::<ReaderState<READER>>();
        *size = reader.reader.read(&mut reader.buffer).unwrap_or(0);
        reader.buffer.as_ptr()
    }
    let mut reader_state = Box::new(ReaderState {
        buffer: [0; 4096],
        reader: reader,
    });
    match lua_loadx(
        state,
        reader_callback::<READER>,
        reader_state.as_mut() as *mut ReaderState<READER> as _,
        chunk_name,
        mode,
    ) {
        Status::Ok => Ok(()),
        Status::SyntaxError => {
            let mut len = 0;
            let err = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                tolstring(state, -1, &mut len),
                len,
            ))
            .to_string();
            pop!(state, 1);
            Err(LError::SyntaxError(err))
        }
        Status::MemoryError => {
            let mut len = 0;
            let err = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                tolstring(state, -1, &mut len),
                len,
            ))
            .to_string();
            pop!(state, 1);
            Err(LError::MemoryError(err))
        }
        _ => unreachable_unchecked(),
    }
}
