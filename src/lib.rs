#![allow(non_camel_case_types)]
#![feature(core_intrinsics)]

use std::{ffi::c_void, hint::unreachable_unchecked};

mod loadx;
pub use loadx::loadx;

mod dump;
pub use dump::dump;

#[macro_export]
macro_rules! pop {
    ($L:expr, $n:expr) => {
        $crate::settop($L, -($n) - 1)
    };
}

#[macro_export]
macro_rules! getglobal {
    ($L:expr, $s:expr) => {
        $crate::lua_getfield($L, $crate::GLOBALSINDEX, $s)
    };
}

#[macro_export]
macro_rules! setglobal {
    ($L:expr, $s:expr) => {
        $crate::lua_setfield($L, $crate::GLOBALSINDEX, $s)
    };
}

#[macro_export]
macro_rules! upvalueindex {
    ($index:expr) => {
        (-10002) - ($index)
    };
}

#[macro_export]
macro_rules! cstr {
    ($str:expr) => {
        concat!($str, "\0").as_ptr() as _
    };
}

pub static REGISTRYINDEX: i32 = -10000;
pub static ENVIRONINDEX: i32 = -10001;
pub static GLOBALSINDEX: i32 = -10002;

pub type lua_State = *mut c_void;
pub type lua_CFunction = unsafe extern "C" fn(state: lua_State) -> i32;
pub type lua_Reader =
    unsafe extern "C" fn(state: lua_State, userdata: *mut c_void, size: &mut usize) -> *const u8;
pub type lua_Writer = unsafe extern "C" fn(
    state: lua_State,
    data: *const u8,
    size: usize,
    userdata: *mut c_void,
) -> i32;
pub type Result = std::result::Result<i32, Box<dyn std::error::Error>>;

#[repr(C)]
#[derive(Debug)]
pub enum Status {
    Ok = 0,
    Yield = 1,
    RuntimeError = 2,
    SyntaxError = 3,
    MemoryError = 4,
    Error = 5,
}

#[derive(Debug)]
pub enum LError {
    RuntimeError,
    SyntaxError(String),
    MemoryError(String),
    DumpError(i32),
}

pub enum LoadMode {
    Any,
    Binary,
    Text,
}

// LOL
// This piece of "code" greatly reduces shit that needed to be in `build.rs`
#[cfg(all(target_os = "linux", target_pointer_width = "32"))]
#[link(name = ":lua_shared_srv.so", kind = "dylib")]
extern "C" {}

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
#[link(name = ":lua_shared.so", kind = "dylib")]
extern "C" {}

#[cfg(target_os = "windows")]
#[link(name = "lua_shared", kind = "dylib")]
extern "C" {}

extern "C" {
    /// state manipulation
    #[link_name = "luaL_newstate"]
    pub fn newstate() -> lua_State;
    #[link_name = "lua_close"]
    pub fn close(state: lua_State);
    #[link_name = "lua_newthread"]
    pub fn newthread(state: lua_State) -> lua_State;

    /// basic stack manipulation
    #[link_name = "lua_gettop"]
    pub fn gettop(state: lua_State);
    #[link_name = "lua_settop"]
    pub fn settop(state: lua_State, index: i32);
    #[link_name = "lua_pushvalue"]
    pub fn pushvalue(state: lua_State, index: i32);
    #[link_name = "lua_remove"]
    pub fn remove(state: lua_State, index: i32);
    #[link_name = "lua_insert"]
    pub fn insert(state: lua_State, index: i32);
    #[link_name = "lua_replace"]
    pub fn replace(state: lua_State, index: i32);
    #[link_name = "lua_checkstack"]
    pub fn checkstack(state: lua_State, size: i32) -> bool;

    /// access functions (stack -> C)
    #[link_name = "lua_isnumber"]
    pub fn isnumber(state: lua_State, index: i32) -> bool;
    #[link_name = "lua_isstring"]
    pub fn isstring(state: lua_State, index: i32) -> bool;
    #[link_name = "lua_iscfunction"]
    pub fn iscfunction(state: lua_State, index: i32) -> bool;
    #[link_name = "lua_isuserdata"]
    pub fn isuserdata(state: lua_State, index: i32) -> bool;
    #[link_name = "lua_type"]
    pub fn get_type(state: lua_State, index: i32) -> i32;
    #[link_name = "lua_typename"]
    pub fn typename(state: lua_State, index: i32) -> *const u8;

    #[link_name = "lua_equal"]
    pub fn equal(state: lua_State, index1: i32, index2: i32) -> bool;
    #[link_name = "lua_rawequal"]
    pub fn rawequal(state: lua_State, index1: i32, index2: i32) -> bool;
    #[link_name = "lua_lessthan"]
    pub fn lessthan(state: lua_State, index1: i32, index2: i32) -> bool;

    #[link_name = "lua_tonumber"]
    pub fn tonumber(state: lua_State, index: i32) -> f64;
    #[link_name = "lua_tointeger"]
    pub fn tointeger(state: lua_State, index: i32) -> isize;
    #[link_name = "lua_toboolean"]
    pub fn toboolean(state: lua_State, index: i32) -> bool;
    #[link_name = "lua_tolstring"]
    pub fn tolstring(state: lua_State, index: i32, len: &mut usize) -> *const u8;
    #[link_name = "lua_objlen"]
    pub fn objlen(state: lua_State, index: i32) -> usize;
    #[link_name = "lua_tocfunction"]
    pub fn tocfunction(state: lua_State, index: i32) -> Option<lua_CFunction>;
    #[link_name = "lua_touserdata"]
    pub fn touserdata(state: lua_State, index: i32) -> *mut c_void;
    #[link_name = "lua_tothread"]
    pub fn tothread(state: lua_State, index: i32) -> lua_State;
    #[link_name = "lua_topointer"]
    pub fn topointer(state: lua_State, index: i32) -> *const c_void;

    /// push functions (C -> stack)
    #[link_name = "lua_pushnil"]
    pub fn pushnil(state: lua_State);
    #[link_name = "lua_pushnumber"]
    pub fn pushnumber(state: lua_State, number: f64);
    #[link_name = "lua_pushinteger"]
    pub fn pushinteger(state: lua_State, integer: isize);
    #[link_name = "lua_pushlstring"]
    pub fn pushlstring(state: lua_State, str: *const u8, len: usize);
    #[link_name = "lua_pushstring"]
    pub fn pushstring(state: lua_State, str: *const u8);
    #[link_name = "lua_pushcclosure"]
    pub fn pushcclosure(state: lua_State, func: lua_CFunction, upvalues: i32);
    #[link_name = "lua_pushboolean"]
    pub fn pushboolean(state: lua_State, bool: i32);
    #[link_name = "lua_pushlightuserdata"]
    pub fn pushlightuserdata(state: lua_State, ptr: *const c_void);
    #[link_name = "lua_pushthread"]
    pub fn pushthread(state: lua_State) -> i32;

    /// get functions (Lua -> stack)
    #[link_name = "lua_gettable"]
    pub fn gettable(state: lua_State, index: i32);
    #[link_name = "lua_getfield"]
    pub fn getfield(state: lua_State, index: i32, str: *const u8);
    #[link_name = "lua_rawget"]
    pub fn rawget(state: lua_State, index: i32);
    #[link_name = "lua_rawgeti"]
    pub fn rawgeti(state: lua_State, index: i32, slot: i32);
    #[link_name = "lua_createtable"]
    pub fn createtable(state: lua_State, array: i32, hash: i32);
    #[link_name = "lua_newuserdata"]
    pub fn newuserdata(state: lua_State, size: usize) -> *mut c_void;
    #[link_name = "lua_getmetatable"]
    pub fn getmetatable(state: lua_State, index: i32) -> i32;
    #[link_name = "lua_getfenv"]
    pub fn getfenv(state: lua_State, index: i32);

    /// set functions (stack -> Lua)
    #[link_name = "lua_settable"]
    pub fn settable(state: lua_State, index: i32);
    #[link_name = "lua_setfield"]
    pub fn setfield(state: lua_State, index: i32, str: *const u8);
    #[link_name = "lua_rawset"]
    pub fn rawset(state: lua_State, index: i32);
    #[link_name = "lua_rawseti"]
    pub fn rawseti(state: lua_State, index: i32, slot: i32);
    #[link_name = "lua_setmetatable"]
    pub fn setmetatable(state: lua_State, index: i32) -> i32;
    #[link_name = "lua_setfenv"]
    pub fn setfenv(state: lua_State, index: i32) -> i32;

    /// `load' and `call' functions (load and run Lua code)
    #[link_name = "lua_call"]
    pub fn call(state: lua_State, nargs: i32, nrets: i32);
    #[link_name = "lua_pcall"]
    pub fn pcall(state: lua_State, nargs: i32, nrets: i32, errfunc: i32) -> Status;
    fn lua_loadx(
        state: lua_State,
        reader: lua_Reader,
        userdata: *mut c_void,
        chunk_name: *const u8,
        mode: *const u8,
    ) -> Status;
    fn lua_dump(state: lua_State, writer: lua_Writer, userdata: *mut c_void) -> i32;

    /// miscellaneous functions
    #[link_name = "lua_error"]
    fn lua_error(state: lua_State) -> i32;
    // #[link_name = "lua_next"]
    // fn lua_next(state: lua_State, index: i32) -> i32;
    // #[link_name = "lua_concat"]
    // fn lua_concat(state: lua_State, index: i32);

    /// lauxlib
    #[link_name = "luaL_checklstring"]
    pub fn Lchecklstring(state: lua_State, index: i32, length: &mut usize) -> *const u8;
    #[link_name = "luaL_optlstring"]
    pub fn Loptlstring(
        state: lua_State,
        index: i32,
        default: *const u8,
        length: &mut usize,
    ) -> *const u8;
    #[link_name = "luaL_checknumber"]
    pub fn Lchecknumber(state: lua_State, index: i32) -> f64;
    #[link_name = "luaL_optnumber"]
    pub fn Loptnumber(state: lua_State, index: i32, default: f64) -> f64;
    #[link_name = "luaL_checkinteger"]
    pub fn Lcheckinteger(state: lua_State, index: i32) -> isize;
    #[link_name = "luaL_optinteger"]
    pub fn Loptinteger(state: lua_State, index: i32, default: isize) -> isize;

    #[link_name = "luaL_checkstack"]
    pub fn Lcheckstack(state: lua_State, size: i32, msg: *const u8);
    #[link_name = "luaL_checktype"]
    pub fn Lchecktype(state: lua_State, index: i32, typ: i32);
    #[link_name = "luaL_checkany"]
    pub fn Lcheckany(state: lua_State, index: i32);

    #[link_name = "luaL_newmetatable"]
    pub fn Lnewmetatable(state: lua_State, type_name: *const u8) -> bool;
    #[link_name = "luaL_checkudata"]
    pub fn Lcheckudata(state: lua_State, index: i32, type_name: *const u8) -> *mut c_void;

    #[link_name = "luaL_where"]
    pub fn Lpush_where(state: lua_State, level: i32);
    #[link_name = "luaL_error"]
    pub fn Lerror(state: lua_State, fmt: *const u8, ...) -> i32;
}

unsafe fn create_callback<FUNC>(state: lua_State, callback: FUNC)
where
    FUNC: 'static + FnMut(lua_State) -> Result,
{
    unsafe extern "C" fn call_callback<FUNC>(state: lua_State) -> i32
    where
        FUNC: 'static + FnMut(lua_State) -> Result,
    {
        let callback_ptr = touserdata(state, upvalueindex!(1));
        match (&mut *callback_ptr.cast::<FUNC>())(state) {
            Ok(nrets) => nrets,
            Err(err) => {
                let error = err.to_string();
                std::mem::drop(err);
                pushlstring(state, error.as_bytes().as_ptr(), error.as_bytes().len());
                std::mem::drop(error);
                lua_error(state);
                unreachable_unchecked();
            }
        }
    }

    unsafe extern "C" fn cleanup_callback<FUNC>(state: lua_State) -> i32
    where
        FUNC: 'static + FnMut(lua_State) -> Result,
    {
        let callback_ptr = touserdata(state, 1);
        let _ = callback_ptr.cast::<FUNC>().read();
        0
    }

    let udata_ptr = newuserdata(state, std::mem::size_of::<FUNC>()).cast::<FUNC>();
    udata_ptr.write(callback);

    createtable(state, 0, 1);
    pushcclosure(state, cleanup_callback::<FUNC>, 0);
    setfield(state, -2, cstr!("__gc"));
    setmetatable(state, -2);
    pushcclosure(state, call_callback::<FUNC>, 1);
}

/// Adds pure rust function as a callback to lua.
/// # Example
/// ```
/// let state = newstate();
/// createtable(state, 0, 2);
/// pushfunction(state, |_| {
///     println!("Hello there!");
///     Ok(0)
/// });
/// setfield(state, -2, cstr!("test_immutable_closure"));
/// fn test_function(state: lua_State) -> Result {
///     println!("Hello there, but from functuin, I guess.");
///     Ok(0)
/// }
/// pushfunction(state, test_function);
/// setfield(state, -2, cstr!("test_immutable_function"));
/// 
/// let mut counter = 0;
/// pushfunction_mut(state, move |_| {
///     println!("Here is yout counter!: {}", counter);
///     pushinteger(state, counter);
///     counter += 1;
///     Ok(1)
/// });
/// setfield(state, -2, cstr!("test_mutable_closure"));
/// setfield(state, GLOBALSINDEX, cstr!("tests"));
/// ```
pub unsafe fn pushfunction<FUNC>(state: lua_State, func: FUNC)
where
    FUNC: 'static + FnMut(lua_State) -> Result,
{
    create_callback(state, func);
}
