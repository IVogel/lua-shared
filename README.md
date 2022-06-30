
# lua-shared
Really simple wrapper around lua_shared(\_srv\_) that tries to not fuck your brains.

Example usecase:
```rust
use lua_shared as lua;
use lua::cstr;
use std::ffi::c_void;

#[no_mangle]
unsafe extern "C" fn fngmod13_open(state: *mut c_void) -> i32 {
	lua::createtable(state, 0, 2);
	lua::pushfunction(state, |_| {
		println!("Hello there!");
		Ok(0)
	});
	setfield(state, -2, cstr!("test_immutable_closure"));
	fn  test_function(state: lua_State) -> Result {
		println!("Hello there, but from functuin, I guess.");
		Ok(0)
	}
	pushfunction(state, test_function);
	setfield(state, -2, cstr!("test_immutable_function"));

	let  mut  counter = 0;
	pushfunction_mut(state, move |_| {
		println!("Here is your counter!: {}", counter);
		pushinteger(state, counter);
		counter += 1;
		Ok(1)
	});
	setfield(state, -2, cstr!("test_mutable_closure"));
	setfield(state, lua::GLOBALSINDEX, cstr!("tests"));
}
```
