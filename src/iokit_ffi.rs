//! Raw IOKit FFI declarations for Thunderbolt device monitoring.

#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]

use std::ffi::{c_char, c_void};

use core_foundation_sys::base::CFAllocatorRef;
use core_foundation_sys::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation_sys::runloop::CFRunLoopSourceRef;

pub type mach_port_t = u32;
pub type io_object_t = mach_port_t;
pub type io_iterator_t = io_object_t;
pub type io_service_t = io_object_t;
pub type kern_return_t = i32;
pub type IONotificationPortRef = *mut c_void;

pub const KERN_SUCCESS: kern_return_t = 0;
pub const kIOMasterPortDefault: mach_port_t = 0;
pub const kIOFirstMatchNotification: &[u8] = b"IOServiceMatched\0";
pub const kIOTerminatedNotification: &[u8] = b"IOServiceTerminate\0";

pub type IOServiceMatchingCallback =
    unsafe extern "C" fn(refcon: *mut c_void, iterator: io_iterator_t);

unsafe extern "C" {
    pub fn IONotificationPortCreate(master_port: mach_port_t) -> IONotificationPortRef;
    pub fn IONotificationPortGetRunLoopSource(notify: IONotificationPortRef) -> CFRunLoopSourceRef;
    pub fn IONotificationPortDestroy(notify: IONotificationPortRef);
    pub fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;

    pub fn IOServiceAddMatchingNotification(
        notifyPort: IONotificationPortRef,
        notificationType: *const c_char,
        matching: CFDictionaryRef,
        callback: IOServiceMatchingCallback,
        refCon: *mut c_void,
        notification: *mut io_iterator_t,
    ) -> kern_return_t;

    pub fn IOServiceGetMatchingServices(
        master_port: mach_port_t,
        matching: CFDictionaryRef,
        existing: *mut io_iterator_t,
    ) -> kern_return_t;

    pub fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;

    pub fn IORegistryEntryCreateCFProperties(
        entry: io_service_t,
        properties: *mut CFMutableDictionaryRef,
        allocator: CFAllocatorRef,
        options: u32,
    ) -> kern_return_t;

    pub fn IOObjectRelease(object: io_object_t) -> kern_return_t;
}
