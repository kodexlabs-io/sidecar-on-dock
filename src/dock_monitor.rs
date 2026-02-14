//! IOKit-based Thunderbolt dock monitoring.
//!
//! Registers for `kIOFirstMatchNotification` and `kIOTerminatedNotification`
//! on `IOThunderboltSwitch` services, then enters a `CFRunLoop`.

use std::cell::Cell;
use std::ffi::{c_char, c_void};
use std::ptr;

use core_foundation::base::{kCFAllocatorDefault, TCFType};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::base::CFRelease;
use core_foundation_sys::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation_sys::runloop::{
    CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun, kCFRunLoopDefaultMode,
};

use crate::iokit_ffi::*;
use crate::sidecar;

const TB_SWITCH_CLASS: &[u8] = b"IOThunderboltSwitch\0";

struct MonitorContext {
    dock_uid: u64,
    ipad_name: Option<String>,
    sidecar_active: Cell<bool>,
}

/// Start monitoring for the configured dock and block on the `CFRunLoop`. Never returns.
pub fn run(dock_uid: u64, ipad_name: Option<String>) -> ! {
    if !sidecar::ensure_loaded() {
        log::error!("Cannot proceed without SidecarCore");
        std::process::exit(1);
    }

    let ctx = Box::leak(Box::new(MonitorContext {
        dock_uid,
        ipad_name,
        sidecar_active: Cell::new(false),
    }));
    let refcon: *mut c_void = (ctx as *mut MonitorContext).cast();

    unsafe {
        let notify_port = IONotificationPortCreate(kIOMasterPortDefault);
        if notify_port.is_null() {
            log::error!("IONotificationPortCreate failed");
            std::process::exit(1);
        }

        let rls = IONotificationPortGetRunLoopSource(notify_port);
        CFRunLoopAddSource(CFRunLoopGetCurrent(), rls, kCFRunLoopDefaultMode);

        let mut connect_iter: io_iterator_t = 0;
        let match_connect = IOServiceMatching(TB_SWITCH_CLASS.as_ptr() as *const c_char);
        let kr = IOServiceAddMatchingNotification(
            notify_port,
            kIOFirstMatchNotification.as_ptr() as *const c_char,
            match_connect as CFDictionaryRef,
            device_appeared,
            refcon,
            &mut connect_iter,
        );
        if kr != KERN_SUCCESS {
            log::error!("IOServiceAddMatchingNotification (connect) failed: {kr}");
            std::process::exit(1);
        }
        drain_iterator(ctx, connect_iter, true);

        let mut disconnect_iter: io_iterator_t = 0;
        let match_disconnect = IOServiceMatching(TB_SWITCH_CLASS.as_ptr() as *const c_char);
        let kr = IOServiceAddMatchingNotification(
            notify_port,
            kIOTerminatedNotification.as_ptr() as *const c_char,
            match_disconnect as CFDictionaryRef,
            device_removed,
            refcon,
            &mut disconnect_iter,
        );
        if kr != KERN_SUCCESS {
            log::error!("IOServiceAddMatchingNotification (disconnect) failed: {kr}");
            std::process::exit(1);
        }
        drain_iterator(ctx, disconnect_iter, false);

        log::info!("Monitoring for Thunderbolt dock UID 0x{:016X}. Entering run loop...", dock_uid);
        CFRunLoopRun();
    }

    unreachable!("CFRunLoopRun returned");
}

unsafe extern "C" fn device_appeared(refcon: *mut c_void, iterator: io_iterator_t) {
    unsafe {
        let ctx = &*(refcon as *const MonitorContext);
        drain_iterator(ctx, iterator, true);
    }
}

unsafe extern "C" fn device_removed(refcon: *mut c_void, iterator: io_iterator_t) {
    unsafe {
        let ctx = &*(refcon as *const MonitorContext);
        drain_iterator(ctx, iterator, false);
    }
}

/// Drain an IOKit iterator, checking each service's UID against the configured dock.
///
/// The iterator **must** be fully drained for IOKit to re-arm the notification.
fn drain_iterator(ctx: &MonitorContext, iterator: io_iterator_t, connected: bool) {
    loop {
        let service = unsafe { IOIteratorNext(iterator) };
        if service == 0 {
            break;
        }

        if let Some(uid) = read_uid(service) {
            log::debug!(
                "Thunderbolt switch {} – UID 0x{:016X}",
                if connected { "appeared" } else { "removed" },
                uid,
            );

            if uid == ctx.dock_uid {
                if connected {
                    log::info!("Dock connected (UID 0x{:016X}). Starting Sidecar...", uid);
                    sidecar::connect(ctx.ipad_name.as_deref());
                    ctx.sidecar_active.set(true);
                } else {
                    log::info!("Dock disconnected (UID 0x{:016X}). Stopping Sidecar...", uid);
                    sidecar::disconnect(ctx.ipad_name.as_deref());
                    ctx.sidecar_active.set(false);
                }
            }
        } else if !connected && ctx.sidecar_active.get() {
            log::info!("Thunderbolt switch removed (UID unreadable). Disconnecting Sidecar as precaution.");
            sidecar::disconnect(ctx.ipad_name.as_deref());
            ctx.sidecar_active.set(false);
        }

        unsafe { IOObjectRelease(service) };
    }
}

/// Read the `"UID"` property (SInt64) from an IORegistry entry.
fn read_uid(service: io_service_t) -> Option<u64> {
    unsafe {
        let mut props_ref: core_foundation_sys::dictionary::CFMutableDictionaryRef = ptr::null_mut();
        let kr = IORegistryEntryCreateCFProperties(service, &mut props_ref, kCFAllocatorDefault as _, 0);
        if kr != KERN_SUCCESS || props_ref.is_null() {
            return None;
        }

        let uid_key = CFString::new("UID");
        let raw_val = CFDictionaryGetValue(props_ref as CFDictionaryRef, uid_key.as_CFTypeRef());

        let result = if raw_val.is_null() {
            None
        } else {
            let number = CFNumber::wrap_under_get_rule(raw_val as core_foundation_sys::number::CFNumberRef);
            number.to_i64().map(|v| v as u64)
        };

        CFRelease(props_ref as *const c_void);
        result
    }
}
