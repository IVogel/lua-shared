#![allow(non_camel_case_types)]

use std::{ffi::c_void, ptr::null_mut};

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
        $crate::getfield($L, $crate::GLOBALSINDEX, $s)
    };
}

#[macro_export]
macro_rules! setglobal {
    ($L:expr, $s:expr) => {
        $crate::setfield($L, $crate::GLOBALSINDEX, $s)
    };
}

#[macro_export]
macro_rules! upvalueindex {
    ($index:expr) => {
        $crate::GLOBALSINDEX - ($index)
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
    // state manipulation

    /// Creates a new Lua state.
    /// It calls `lua_newstate` with an allocator based on the standard C realloc function and then sets a panic function that prints an error message to the standard error output in case of fatal errors.
    ///
    /// Returns the new state, or `NULL` if there is a memory allocation error.
    #[link_name = "luaL_newstate"]
    pub fn newstate() -> lua_State;
    /// Destroys all objects in the given Lua state (calling the corresponding garbage-collection metamethods, if any) and frees all dynamic memory used by this state.
    /// In several platforms, you may not need to call this function, because all resources are naturally released when the host program ends.
    /// On the other hand, long-running programs that create multiple states, such as daemons or web servers, will probably need to close states as soon as they are not needed.
    #[link_name = "lua_close"]
    pub fn close(state: lua_State);
    /// Creates a new thread, pushes it on the stack, and returns a pointer to a [`lua_State`] that represents this new thread.
    /// The new thread returned by this function shares with the original thread its global environment, but has an independent execution stack.
    /// There is no explicit function to close or to destroy a thread. Threads are subject to garbage collection, like any Lua object.
    #[link_name = "lua_newthread"]
    pub fn newthread(state: lua_State) -> lua_State;

    // basic stack manipulation

    /// Returns the index of the top element in the stack. Because indices start at 1, this result is equal to the number of elements in the stack; in particular, 0 means an empty stack.
    #[link_name = "lua_gettop"]
    pub fn gettop(state: lua_State);
    /// Accepts any index, or 0, and sets the stack top to this index.
    /// If the new top is larger than the old one, then the new elements are filled with **nil**. If index is 0, then all stack elements are removed.
    #[link_name = "lua_settop"]
    pub fn settop(state: lua_State, index: i32);
    /// Pushes a copy of the element at the given index onto the stack.
    #[link_name = "lua_pushvalue"]
    pub fn pushvalue(state: lua_State, index: i32);
    /// Removes the element at the given valid index, shifting down the elements above this index to fill the gap.
    /// This function cannot be called with a pseudo-index, because a pseudo-index is not an actual stack position.
    #[link_name = "lua_remove"]
    pub fn remove(state: lua_State, index: i32);
    /// Moves the top element into the given valid index, shifting up the elements above this index to open space.
    /// This function cannot be called with a pseudo-index, because a pseudo-index is not an actual stack position.
    #[link_name = "lua_insert"]
    pub fn insert(state: lua_State, index: i32);
    /// Moves the top element into the given valid index without shifting any element (therefore replacing the value at that given index), and then pops the top element.
    #[link_name = "lua_replace"]
    pub fn replace(state: lua_State, index: i32);
    /// Ensures that the stack has space for at least n extra slots (that is, that you can safely push up to n values into it).
    /// It returns false if it cannot fulfill the request, either because it would cause the stack to be larger than a fixed maximum size (typically at least several thousand elements) or because it cannot allocate memory for the extra space.
    /// This function never shrinks the stack; if the stack already has space for the extra slots, it is left unchanged.
    #[link_name = "lua_checkstack"]
    pub fn checkstack(state: lua_State, size: i32) -> bool;

    // access functions (stack -> C)

    /// Returns `true` if the value at the given index is a number or a string convertible to a number, and `false` otherwise.
    #[link_name = "lua_isnumber"]
    pub fn isnumber(state: lua_State, index: i32) -> bool;
    /// Returns `true` if the value at the given index is a string or a number (which is always convertible to a string), and `false` otherwise.
    #[link_name = "lua_isstring"]
    pub fn isstring(state: lua_State, index: i32) -> bool;
    /// Returns `true` if the value at the given index is a C function, and `true` otherwise.
    #[link_name = "lua_iscfunction"]
    pub fn iscfunction(state: lua_State, index: i32) -> bool;
    /// Returns `true` if the value at the given index is a userdata (either full or light), and `false` otherwise.
    #[link_name = "lua_isuserdata"]
    pub fn isuserdata(state: lua_State, index: i32) -> bool;
    /// Returns the type of the value in the given valid index, or `LUA_TNONE` for a non-valid (but acceptable) index.
    /// The types returned by [`get_type`] (`lua_type`) are coded by the following constants defined in lua.h: `LUA_TNIL` (0), `LUA_TNUMBER, LUA_TBOOLEAN, LUA_TSTRING, LUA_TTABLE, LUA_TFUNCTION, LUA_TUSERDATA, LUA_TTHREAD,` and `LUA_TLIGHTUSERDATA`.
    #[link_name = "lua_type"]
    pub fn get_type(state: lua_State, index: i32) -> i32;
    /// Returns the name of the type encoded by the value tp, which must be one the values returned by [`get_type`] (`lua_type`).
    #[link_name = "lua_typename"]
    pub fn typename(state: lua_State, index: i32) -> *const u8;

    /// Returns `true` if the two values in acceptable indices `index1` and `index2` are equal, following the semantics of the Lua `==` operator (that is, may call metamethods).
    /// Otherwise returns `false`. Also returns `false` if any of the indices is non valid.
    #[link_name = "lua_equal"]
    pub fn equal(state: lua_State, index1: i32, index2: i32) -> bool;
    /// Returns `true` if the two values in acceptable indices `index1` and `index2` are primitively equal (that is, without calling metamethods). Otherwise returns `false`.
    /// Also returns `false` if any of the indices are non valid.
    #[link_name = "lua_rawequal"]
    pub fn rawequal(state: lua_State, index1: i32, index2: i32) -> bool;
    /// Returns `true` if the value at acceptable index `index1` is smaller than the value at acceptable index `index2`, following the semantics of the Lua `<` operator (that is, may call metamethods).
    /// Otherwise returns `false`. Also returns 0 if any of the indices is non valid.
    #[link_name = "lua_lessthan"]
    pub fn lessthan(state: lua_State, index1: i32, index2: i32) -> bool;

    /// Converts the Lua value at the given acceptable index to the C type `lua_Number` (see [lua_Number](https://www.lua.org/manual/5.1/manual.html#lua_Number)).
    /// The Lua value must be a number or a string convertible to a number (see [§2.2.1](https://www.lua.org/manual/5.1/manual.html#2.2.1)); otherwise, [`tonumber`] (`lua_tonumber`) returns `0`.
    #[link_name = "lua_tonumber"]
    pub fn tonumber(state: lua_State, index: i32) -> f64;

    /// Converts the Lua value at the given acceptable index to the signed integral type [lua_Integer](https://www.lua.org/manual/5.1/manual.html#lua_Integer).
    /// The Lua value must be a number or a string convertible to a number (see [§2.2.1](https://www.lua.org/manual/5.1/manual.html#2.2.1)); otherwise, [`tointeger`] (`lua_tointeger`) returns 0.
    #[link_name = "lua_tointeger"]
    pub fn tointeger(state: lua_State, index: i32) -> isize;
    /// Converts the Lua value at the given acceptable index to a C boolean value (0 or 1).
    /// Like all tests in Lua, [`toboolean`] (`lua_toboolean`) returns `true` for any Lua value different from **false** and **nil**; otherwise it returns `false`.
    /// It also returns `false` when called with a non-valid index.
    #[link_name = "lua_toboolean"]
    pub fn toboolean(state: lua_State, index: i32) -> bool;
    /// Converts the Lua value at the given acceptable index to a C string.
    /// If `len` is not `NULL`, it also sets `*len` with the string length.
    /// The Lua value must be a string or a number; otherwise, the function returns `NULL`.
    /// If the value is a number, then [`tolstring`] also changes the actual value in the stack to a string.
    #[link_name = "lua_tolstring"]
    pub fn tolstring(state: lua_State, index: i32, len: &mut usize) -> *const u8;
    /// Returns the "length" of the value at the given acceptable index: for strings, this is the string length; for tables, this is the result of the length operator (`'#'`); for userdata, this is the size of the block of memory allocated for the userdata; for other values, it is 0.
    #[link_name = "lua_objlen"]
    pub fn objlen(state: lua_State, index: i32) -> usize;
    /// Converts a value at the given acceptable index to a C function. That value must be a C function; otherwise, returns `NULL`.
    #[link_name = "lua_tocfunction"]
    pub fn tocfunction(state: lua_State, index: i32) -> Option<lua_CFunction>;
    /// If the value at the given acceptable index is a full userdata, returns its block address. If the value is a light userdata, returns its pointer. Otherwise, returns `NULL`.
    #[link_name = "lua_touserdata"]
    pub fn touserdata(state: lua_State, index: i32) -> *mut c_void;
    /// Converts the value at the given acceptable index to a Lua thread (represented as [`lua_State`]). This value must be a thread; otherwise, the function returns `NULL`.
    #[link_name = "lua_tothread"]
    pub fn tothread(state: lua_State, index: i32) -> lua_State;
    /// Converts the value at the given acceptable index to a generic C pointer (`void*`).
    /// The value can be a userdata, a table, a thread, or a function; otherwise, [`topointer`] (`lua_topointer`) returns `NULL`.
    /// Different objects will give different pointers. There is no way to convert the pointer back to its original value.
    ///
    /// Typically this function is used only for debug information.
    #[link_name = "lua_topointer"]
    pub fn topointer(state: lua_State, index: i32) -> *const c_void;

    // push functions (C -> stack)

    /// Pushes a nil value onto the stack.
    #[link_name = "lua_pushnil"]
    pub fn pushnil(state: lua_State);
    /// Pushes a number with value `number` onto the stack.
    #[link_name = "lua_pushnumber"]
    pub fn pushnumber(state: lua_State, number: f64);
    /// Pushes a number with value `number` onto the stack.
    #[link_name = "lua_pushinteger"]
    pub fn pushinteger(state: lua_State, integer: isize);
    /// Pushes the string pointed to by `str` with size `len` onto the stack.
    /// Lua makes (or reuses) an internal copy of the given string, so the memory at `str` can be freed or reused immediately after the function returns.
    /// The string can contain embedded zeros.
    #[link_name = "lua_pushlstring"]
    pub fn pushlstring(state: lua_State, str: *const u8, len: usize);
    /// Pushes the zero-terminated string pointed to by `str` onto the stack. Lua makes (or reuses) an internal copy of the given string, so the memory at `str` can be freed or reused immediately after the function returns.
    /// The string cannot contain embedded zeros; it is assumed to end at the first zero.
    #[link_name = "lua_pushstring"]
    pub fn pushstring(state: lua_State, str: *const u8);
    /// Pushes a new C closure onto the stack.
    ///
    /// When a C function is created, it is possible to associate some values with it, thus creating a C closure (see [§3.4](https://www.lua.org/manual/5.1/manual.html#3.4)); these values are then accessible to the function whenever it is called.
    /// To associate values with a C function, first these values should be pushed onto the stack (when there are multiple values, the first value is pushed first).
    /// Then [`pushcclosure`] (`lua_pushcclosure`)  is called to create and push the C function onto the stack, with the argument n telling how many values should be associated with the function.
    /// [`pushcclosure`] (`lua_pushcclosure`) also pops these values from the stack.
    ///
    /// The maximum value for `upvalues` is 255.
    #[link_name = "lua_pushcclosure"]
    pub fn pushcclosure(state: lua_State, func: lua_CFunction, upvalues: i32);
    /// Pushes a boolean value with value `bool` onto the stack.
    #[link_name = "lua_pushboolean"]
    pub fn pushboolean(state: lua_State, bool: i32);
    /// Pushes a light userdata onto the stack.
    ///
    /// Userdata represent C values in Lua
    /// A _light userdata_ represents a pointer.
    /// It is a value (like a number): you do not create it, it has no individual metatable, and it is not collected (as it was never created).
    /// A light userdata is equal to "any" light userdata with the same C address.
    #[link_name = "lua_pushlightuserdata"]
    pub fn pushlightuserdata(state: lua_State, ptr: *const c_void);
    /// Pushes the thread represented by `state` onto the stack. Returns 1 if this thread is the main thread of its state.
    #[link_name = "lua_pushthread"]
    pub fn pushthread(state: lua_State) -> i32;

    // get functions (Lua -> stack)

    /// Pushes onto the stack the value `t[k]`, where `t` is the value at the given valid index and `k` is the value at the top of the stack.
    ///
    /// This function pops the key from the stack (putting the resulting value in its place).
    /// As in Lua, this function may trigger a metamethod for the "index" event (see [§2.8](https://www.lua.org/manual/5.1/manual.html#2.8)).
    #[link_name = "lua_gettable"]
    pub fn gettable(state: lua_State, index: i32);
    /// Pushes onto the stack the value `t[k]`, where `t` is the value at the given valid index. As in Lua, this function may trigger a metamethod for the "index" event (see [§2.8](https://www.lua.org/manual/5.1/manual.html#2.8)).
    #[link_name = "lua_getfield"]
    pub fn getfield(state: lua_State, index: i32, str: *const u8);
    /// Similar to [`gettable`] (`lua_gettable`), but does a raw access (i.e., without metamethods).
    #[link_name = "lua_rawget"]
    pub fn rawget(state: lua_State, index: i32);
    /// Pushes onto the stack the value `t[n]`, where `t` is the value at the given valid index. The access is raw; that is, it does not invoke metamethods.
    #[link_name = "lua_rawgeti"]
    pub fn rawgeti(state: lua_State, index: i32, slot: i32);
    /// Creates a new empty table and pushes it onto the stack.
    /// The new table has space pre-allocated for `array` array elements and `hash` non-array elements.
    /// This pre-allocation is useful when you know exactly how many elements the table will have.
    #[link_name = "lua_createtable"]
    pub fn createtable(state: lua_State, array: i32, hash: i32);
    /// This function allocates a new block of memory with the given size, pushes onto the stack a new full userdata with the block address, and returns this address.
    ///
    /// Userdata represent C values in Lua.
    /// A _full userdata_ represents a block of memory.
    /// It is an object (like a table): you must create it, it can have its own metatable, and you can detect when it is being collected.
    /// A full userdata is only equal to itself (under raw equality).
    ///
    /// When Lua collects a full userdata with a [`__gc`](https://www.lua.org/manual/5.1/manual.html#2.10.1) metamethod, Lua calls the metamethod and marks the userdata as finalized.
    /// When this userdata is collected again then Lua frees its corresponding memory.
    #[link_name = "lua_newuserdata"]
    pub fn newuserdata(state: lua_State, size: usize) -> *mut c_void;
    /// Pushes onto the stack the metatable of the value at the given acceptable index. If the index is not valid, or if the value does not have a metatable, the function returns 0 and pushes nothing on the stack.
    #[link_name = "lua_getmetatable"]
    pub fn getmetatable(state: lua_State, index: i32) -> i32;
    /// Pushes onto the stack the environment table of the value at the given index.
    #[link_name = "lua_getfenv"]
    pub fn getfenv(state: lua_State, index: i32);

    // set functions (stack -> Lua)

    /// Does the equivalent to `t[k] = v`, where `t` is the value at the given valid index, `v` is the value at the top of the stack, and `k` is the value just below the top.
    ///
    /// This function pops both the key and the value from the stack. As in Lua, this function may trigger a metamethod for the "newindex" event (see [§2.8](https://www.lua.org/manual/5.1/manual.html#2.8)).
    #[link_name = "lua_settable"]
    pub fn settable(state: lua_State, index: i32);
    /// Does the equivalent to `t[k] = v`, where `t` is the value at the given valid index and `v` is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. As in Lua, this function may trigger a metamethod for the "newindex" event (see [§2.8](https://www.lua.org/manual/5.1/manual.html#2.8)).
    #[link_name = "lua_setfield"]
    pub fn setfield(state: lua_State, index: i32, str: *const u8);
    /// Similar to [`settable`] (`lua_settable`), but does a raw assignment (i.e., without metamethods).
    #[link_name = "lua_rawset"]
    pub fn rawset(state: lua_State, index: i32);
    /// Does the equivalent of `t[n] = v`, where `t` is the value at the given valid index and `v` is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. The assignment is raw; that is, it does not invoke metamethods.
    #[link_name = "lua_rawseti"]
    pub fn rawseti(state: lua_State, index: i32, slot: i32);
    /// Pops a table from the stack and sets it as the new metatable for the value at the given acceptable index.
    #[link_name = "lua_setmetatable"]
    pub fn setmetatable(state: lua_State, index: i32) -> i32;
    /// Pops a table from the stack and sets it as the new environment for the value at the given index. If the value at the given index is neither a function nor a thread nor a userdata, [`setfenv`] (`lua_setfenv`) returns 0. Otherwise it returns 1.
    #[link_name = "lua_setfenv"]
    pub fn setfenv(state: lua_State, index: i32) -> i32;

    // `load' and `call' functions (load and run Lua code)

    /// Calls a function.
    ///
    /// To call a function you must use the following protocol: first, the function to be called is pushed onto the stack; then, the arguments to the function are pushed in direct order; that is, the first argument is pushed first.
    /// Finally you call [`call`] (`lua_call`); `nargs` is the number of arguments that you pushed onto the stack.
    /// All arguments and the function value are popped from the stack when the function is called.
    /// The function results are pushed onto the stack when the function returns.
    /// The number of results is adjusted to `nrets`, unless `nrets` is `LUA_MULTRET`.
    /// In this case, _all_ results from the function are pushed.
    /// Lua takes care that the returned values fit into the stack space.
    /// The function results are pushed onto the stack in direct order (the first result is pushed first), so that after the call the last result is on the top of the stack.
    ///
    /// Any error inside the called function is propagated upwards (with a `longjmp`).
    #[link_name = "lua_call"]
    pub fn call(state: lua_State, nargs: i32, nrets: i32);
    /// Calls a function in protected mode.
    ///
    /// Both nargs and nresults have the same meaning as in [`call`] (`lua_call`).
    /// If there are no errors during the call, [`pcall`] (`lua_pcall`) behaves exactly like [`call`] (`lua_call`).
    /// However, if there is any error, [`pcall`] (`lua_pcall`) catches it, pushes a single value on the stack (the error message), and returns an error code.
    /// Like [`call`] (`lua_call`), [`pcall`] (`lua_pcall`) always removes the function and its arguments from the stack.
    ///
    /// If `errfunc` is 0, then the error message returned on the stack is exactly the original error message.
    /// Otherwise, `errfunc` is the stack index of an _error handler function_. (In the current implementation, this index cannot be a pseudo-index.)
    /// In case of runtime errors, this function will be called with the error message and its return value will be the message returned on the stack by [`pcall`] (`lua_pcall`).
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

    // miscellaneous functions

    /// Generates a Lua error. The error message (which can actually be a Lua value of any type) must be on the stack top. This function does a long jump, and therefore never returns. (see [`luaL_error`](https://www.lua.org/manual/5.1/manual.html#luaL_error)).
    #[link_name = "lua_error"]
    pub fn error(state: lua_State) -> !;
    // #[link_name = "lua_next"]
    // fn lua_next(state: lua_State, index: i32) -> i32;
    // #[link_name = "lua_concat"]
    // fn lua_concat(state: lua_State, index: i32);

    // lauxlib

    /// Checks whether the function argument `index` is a string and returns this string; if `length` is not NULL fills `*length` with the string's length.
    ///
    /// This function uses [`tolstring`] (`lua_tolstring`) to get its result, so all conversions and caveats of that function apply here.
    #[link_name = "luaL_checklstring"]
    pub fn Lchecklstring(state: lua_State, index: i32, length: &mut usize) -> *const u8;
    /// If the function argument `index` is a string, returns this string. If this argument is absent or is **nil**, returns `default`. Otherwise, raises an error.
    ///
    /// If `length` is not `NULL`, fills the position `*length` with the results's length.
    #[link_name = "luaL_optlstring"]
    pub fn Loptlstring(
        state: lua_State,
        index: i32,
        default: *const u8,
        length: &mut usize,
    ) -> *const u8;
    /// Checks whether the function argument `index` is a number and returns this number.
    #[link_name = "luaL_checknumber"]
    pub fn Lchecknumber(state: lua_State, index: i32) -> f64;
    /// If the function argument `index` is a number, returns this number. If this argument is absent or is **nil**, returns `default`. Otherwise, raises an error.
    #[link_name = "luaL_optnumber"]
    pub fn Loptnumber(state: lua_State, index: i32, default: f64) -> f64;
    /// Checks whether the function argument `index` is a number and returns this number cast to a [`lua_Integer`](https://www.lua.org/manual/5.1/manual.html#lua_Integer).
    #[link_name = "luaL_checkinteger"]
    pub fn Lcheckinteger(state: lua_State, index: i32) -> isize;
    /// If the function argument `index` is a number, returns this number cast to a [`lua_Integer`](https://www.lua.org/manual/5.1/manual.html#lua_Integer).
    /// If this argument is absent or is **nil**, returns `default`. Otherwise, raises an error.
    #[link_name = "luaL_optinteger"]
    pub fn Loptinteger(state: lua_State, index: i32, default: isize) -> isize;
    /// Grows the stack size to `top + size` elements, raising an error if the stack cannot grow to that size. `msg` is an additional text to go into the error message.
    #[link_name = "luaL_checkstack"]
    pub fn Lcheckstack(state: lua_State, size: i32, msg: *const u8);
    /// Checks whether the function argument `index` has type `typ`. See [`lua_type`](https://www.lua.org/manual/5.1/manual.html#lua_type) for the encoding of types for `typ`.
    #[link_name = "luaL_checktype"]
    pub fn Lchecktype(state: lua_State, index: i32, typ: i32);
    /// Checks whether the function has an argument of any type (including **nil**) at position `index`.
    #[link_name = "luaL_checkany"]
    pub fn Lcheckany(state: lua_State, index: i32);
    /// Raises an error with the following message, where `func` is retrieved from the call stack
    #[link_name = "luaL_argerror"]
    pub fn Largerror(state: lua_State, index: i32, msg: *const u8) -> !;

    /// If the registry already has the key `type_name`, returns `false`. Otherwise, creates a new table to be used as a metatable for userdata, adds it to the registry with key `type_name`, and returns `true`.
    ///
    /// In both cases pushes onto the stack the final value associated with `type_name` in the registry.
    #[link_name = "luaL_newmetatable"]
    pub fn Lnewmetatable(state: lua_State, type_name: *const u8) -> bool;
    /// Checks whether the function argument `index` is a userdata of the type `type_name` (see [`Lnewmetatable`]).
    #[link_name = "luaL_checkudata"]
    pub fn Lcheckudata(state: lua_State, index: i32, type_name: *const u8) -> *mut c_void;

    /// Pushes onto the stack a string identifying the current position of the control at level `level` in the call stack. Typically this string has the following format:
    /// ```text
    ///     chunkname:currentline:
    /// ```
    /// Level 0 is the running function, level 1 is the function that called the running function, etc.
    ///
    /// This function is used to build a prefix for error messages.
    #[link_name = "luaL_where"]
    pub fn Lpush_where(state: lua_State, level: i32);
    /// Raises an error. The error message format is given by `fmt` plus any extra arguments, following the same rules of [`lua_pushfstring`](https://www.lua.org/manual/5.1/manual.html#lua_pushfstring).
    /// It also adds at the beginning of the message the file name and the line number where the error occurred, if this information is available.
    ///
    /// This function never returns, but it is an idiom to use it in C functions as `return luaL_error(args)`.
    #[link_name = "luaL_error"]
    pub fn Lerror(state: lua_State, fmt: *const u8, ...) -> !;

    /// Loads a buffer as a Lua chunk. This function uses [`lua_load`](https://www.lua.org/manual/5.1/manual.html#lua_load) to load the chunk in the buffer pointed to by `buffer` with size `size`.
    /// 
    /// This function returns the same results as [`lua_load`](https://www.lua.org/manual/5.1/manual.html#lua_load). `name` is the chunk name, used for debug information and error messages. The string mode works as in function [`lua_load`](https://www.lua.org/manual/5.1/manual.html#lua_load). 
    #[link_name = "luaL_loadbufferx"]
    pub fn Lloadbufferx(state: lua_State, buffer: *const u8, size: usize, name: *const u8, mode: *const u8) -> Status;

    /// Opens all standard Lua libraries into the given state.
    #[link_name = "luaL_openlibs"]
    pub fn Lopenlibs(state: lua_State);

    #[link_name = "luaopen_base"]
    pub fn open_base(state: lua_State) -> i32;
    #[link_name = "luaopen_package"]
    pub fn open_package(state: lua_State) -> i32;
    #[link_name = "luaopen_math"]
    pub fn open_math(state: lua_State) -> i32;
    #[link_name = "luaopen_bit"]
    pub fn open_bit(state: lua_State) -> i32;
    #[link_name = "luaopen_string"]
    pub fn open_string(state: lua_State) -> i32;
    #[link_name = "luaopen_table"]
    pub fn open_table(state: lua_State) -> i32;
    #[link_name = "luaopen_os"]
    pub fn open_os(state: lua_State) -> i32;
    #[link_name = "luaopen_debug"]
    pub fn open_debug(state: lua_State) -> i32;
    #[link_name = "luaopen_jit"]
    pub fn open_jit(state: lua_State) -> i32;
}

/// Pushes rust function/closure to lua stack.
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
pub unsafe fn pushfunction<FUNC>(state: lua_State, callback: FUNC)
where
    FUNC: 'static + FnMut(lua_State) -> Result,
{
    unsafe extern "C" fn call_callback<FUNC>(state: lua_State) -> i32
    where
        FUNC: 'static + FnMut(lua_State) -> Result,
    {
        let callback_ptr = if std::mem::size_of::<FUNC>() > 0 {
            touserdata(state, upvalueindex!(1))
        } else {
            null_mut()
        };
        match (&mut *callback_ptr.cast::<FUNC>())(state) {
            Ok(nrets) => nrets,
            Err(err) => {
                let error_str = err.to_string();
                std::mem::drop(err);
                pushlstring(
                    state,
                    error_str.as_bytes().as_ptr(),
                    error_str.as_bytes().len(),
                );
                std::mem::drop(error_str);
                error(state);
            }
        }
    }

    if std::mem::size_of::<FUNC>() > 0 {
        let udata_ptr = newuserdata(state, std::mem::size_of::<FUNC>()).cast::<FUNC>();
        udata_ptr.write(callback);
        unsafe extern "C" fn cleanup_callback<FUNC>(state: lua_State) -> i32
        where
            FUNC: 'static + FnMut(lua_State) -> Result,
        {
            let callback_ptr = touserdata(state, 1);
            let _ = callback_ptr.cast::<FUNC>().read();
            0
        }
        createtable(state, 0, 1);
        pushcclosure(state, cleanup_callback::<FUNC>, 0);
        setfield(state, -2, cstr!("__gc"));
        setmetatable(state, -2);
        pushcclosure(state, call_callback::<FUNC>, 1);
    } else {
        pushcclosure(state, call_callback::<FUNC>, 0);
    }
}
