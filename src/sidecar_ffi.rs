//! Low-level FFI wrappers around the private `SidecarCore.framework`.
//!
//! SidecarCore lives in the dyld shared cache and is loaded via `dlopen`.
//! All interaction happens through the Objective-C runtime using `objc2`.

use std::ffi::{c_char, c_int, c_void};

use block2::StackBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::msg_send;
use objc2_foundation::NSString;

const SIDECAR_FRAMEWORK_PATH: &[u8] =
    b"/System/Library/PrivateFrameworks/SidecarCore.framework/SidecarCore\0";
const RTLD_LAZY: c_int = 0x1;

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
}

/// Load the SidecarCore private framework into the process.
pub fn load_framework() -> Result<(), String> {
    let handle =
        unsafe { dlopen(SIDECAR_FRAMEWORK_PATH.as_ptr() as *const c_char, RTLD_LAZY) };
    if handle.is_null() {
        Err("Failed to load SidecarCore.framework (dlopen returned null)".into())
    } else {
        Ok(())
    }
}

/// Obtain a reference to the `SidecarDisplayManager` ObjC class.
pub fn display_manager_class() -> Option<&'static AnyClass> {
    AnyClass::get(c"SidecarDisplayManager")
}

/// `[SidecarDisplayManager sharedManager]`.
pub unsafe fn shared_manager(cls: &AnyClass) -> Option<Retained<AnyObject>> {
    unsafe { msg_send![cls, sharedManager] }
}

/// `[manager devices]` returning an `NSArray<SidecarDevice>`.
pub unsafe fn devices(manager: &AnyObject) -> Option<Retained<AnyObject>> {
    unsafe { msg_send![manager, devices] }
}

/// `[device name]` returning an `NSString`.
pub unsafe fn device_name(device: &AnyObject) -> Option<Retained<NSString>> {
    unsafe { msg_send![device, name] }
}

/// `[array count]`.
pub unsafe fn array_count(array: &AnyObject) -> usize {
    msg_send![array, count]
}

/// `[array objectAtIndex:index]`.
pub unsafe fn array_object_at(array: &AnyObject, index: usize) -> Option<Retained<AnyObject>> {
    unsafe { msg_send![array, objectAtIndex: index] }
}

/// `[manager connectToDevice:device completion:block]`.
pub unsafe fn connect_to_device(manager: &AnyObject, device: &AnyObject) {
    let block = StackBlock::new(|error: *mut AnyObject| {
        if error.is_null() {
            log::info!("Sidecar connected successfully");
        } else {
            log::error!("Sidecar connection failed (completion reported error)");
        }
    });
    unsafe { msg_send![manager, connectToDevice: device, completion: &*block] }
}

/// `[manager disconnectFromDevice:device completion:block]`.
pub unsafe fn disconnect_from_device(manager: &AnyObject, device: &AnyObject) {
    let block = StackBlock::new(|error: *mut AnyObject| {
        if error.is_null() {
            log::info!("Sidecar disconnected successfully");
        } else {
            log::error!("Sidecar disconnect failed (completion reported error)");
        }
    });
    unsafe { msg_send![manager, disconnectFromDevice: device, completion: &*block] }
}
