use alloc::boxed::Box;

use no_std_framework_core::{Agent, Container};

#[no_mangle]
pub extern "C" fn ffi_container_new() -> *mut Container {
    let container = Container::default();
    Box::into_raw(Box::new(container))
}

// Wrapper to add an agent to the container
#[no_mangle]
pub unsafe extern "C" fn ffi_container_add_agent(container: *mut Container, agent: *mut Agent) {
    if !container.is_null() {
        unsafe {
            (*container).add_agent(*Box::from_raw(agent));
        }
    }
}

// Wrapper to start the container
#[no_mangle]
pub unsafe extern "C" fn ffi_container_start(container: *mut Container) -> i32 {
    if container.is_null() {
        return -1; // Error: Null pointer
    }

    let result = unsafe { Box::from_raw(container).start() };
    match result {
        Ok(_) => 0,   // Success
        Err(_) => -2, // Error
    }
}

// Wrapper to free the container
#[no_mangle]
pub unsafe extern "C" fn ffi_container_free(container: *mut Container) {
    if !container.is_null() {
        unsafe {
            drop(Box::from_raw(container)); // This will free the container
        }
    }
}
