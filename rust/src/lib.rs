use std::{str, sync::Arc};

use nucleo::{Config, Nucleo};

#[repr(C)]
pub struct NucleoHandle {
    internal: Nucleo<Box<[u8]>>,
}

pub type VoidCallbackFn = extern "C" fn();

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_new(cb: VoidCallbackFn) -> *mut NucleoHandle {
    let handle = NucleoHandle {
        internal: Nucleo::new(Config::DEFAULT, Arc::new(move || cb()), None, 1),
    };
    std::ptr::from_mut(Box::leak(Box::new(handle)))
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_destroy(ptr: *mut NucleoHandle) {
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_tick(ptr: *mut NucleoHandle, ms: std::ffi::c_uint) {
    unsafe { (*ptr).internal.tick(ms.into()) };
}

#[repr(C)]
pub struct NucleoDartStringMut {
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
pub struct NucleoDartString {
    pub ptr: *const u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_add(ptr: *mut NucleoHandle, item: NucleoDartStringMut) {
    let reference: &mut NucleoHandle = unsafe { ptr.as_mut().unwrap() };
    let injector = reference.internal.injector();

    let slice_ref = unsafe { std::slice::from_raw_parts_mut(item.ptr, item.len) };
    let slice_boxed: Box<[u8]> = slice_ref.into();

    injector.push(slice_boxed, |v, z| {
        let boxed_str = unsafe { str::from_boxed_utf8_unchecked(v.clone()) };
        z[0] = boxed_str.into();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_add_all(
    ptr: *mut NucleoHandle,
    list: *const NucleoDartStringMut,
    len: usize,
) {
    let items = unsafe { std::slice::from_raw_parts(list, len) };

    let reference: &mut NucleoHandle = unsafe { ptr.as_mut().unwrap() };
    let injector = reference.internal.injector();

    for (_, item) in items.iter().enumerate() {
        let slice_ref = unsafe { std::slice::from_raw_parts_mut(item.ptr, item.len) };
        let slice_boxed: Box<[u8]> = slice_ref.into();

        injector.push(slice_boxed, |v, z| {
            let boxed_str = unsafe { str::from_boxed_utf8_unchecked(v.clone()) };
            z[0] = boxed_str.into();
        });
    }
}

#[repr(C)]
pub enum IsAppend {
    IsAppendYes,
    IsAppendNo,
}

/// By specifying append the caller promises that text passed to the previous reparse invocation
/// is a prefix of new_text. This enables additional optimizations but can lead to missing matches
/// if an incorrect value is passed.
#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_reparse(
    ptr: *mut NucleoHandle,
    new_text: NucleoDartString,
    append: IsAppend,
) {
    let slice_ref = unsafe { std::slice::from_raw_parts(new_text.ptr, new_text.len) };
    let new_text_str: &str = unsafe { std::str::from_utf8_unchecked(slice_ref) };

    let reference: &mut NucleoHandle = unsafe { ptr.as_mut().unwrap() };
    reference.internal.pattern.reparse(
        0,
        new_text_str,
        nucleo::pattern::CaseMatching::Ignore,
        nucleo::pattern::Normalization::Smart,
        match append {
            IsAppend::IsAppendYes => true,
            IsAppend::IsAppendNo => false,
        },
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_get_snapshot(ptr: *mut NucleoHandle) -> *const SnapshotHandle {
    let reference: &mut NucleoHandle = unsafe { ptr.as_mut().unwrap() };
    return std::ptr::from_ref(reference.internal.snapshot());
}

type SnapshotHandle = nucleo::Snapshot<Box<[u8]>>;

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_item_count(handle: *const SnapshotHandle) -> u32 {
    let reference = unsafe { handle.as_ref().unwrap() };
    reference.item_count()
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_matched_item_count(
    handle: *const SnapshotHandle,
) -> u32 {
    let reference = unsafe { handle.as_ref().unwrap() };
    reference.matched_item_count()
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_item(
    handle: *const SnapshotHandle,
    index: u32,
) -> NucleoDartString {
    let reference = unsafe { handle.as_ref().unwrap() };
    let item = unsafe { reference.get_item_unchecked(index) };
    NucleoDartString {
        ptr: item.data.as_ptr(),
        len: item.data.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_matched_item(
    handle: *const SnapshotHandle,
    index: u32,
) -> NucleoDartString {
    let reference = unsafe { handle.as_ref().unwrap() };
    let item = reference.get_matched_item(index).unwrap();
    NucleoDartString {
        ptr: item.data.as_ptr(),
        len: item.data.len(),
    }
}

pub type AppendCallbackFn = extern "C" fn(NucleoDartString);

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_matched_items(
    handle: *const SnapshotHandle,
    start: u32,
    end: u32,
    cb: AppendCallbackFn,
) {
    let reference = unsafe { handle.as_ref().unwrap() };
    reference.matched_items(start..end).for_each(|item| {
        cb(NucleoDartString {
            ptr: item.data.as_ptr(),
            len: item.data.len(),
        });
    });
}
