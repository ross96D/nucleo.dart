use std::{str, sync::Arc};

use nucleo::{Config, Nucleo};

#[repr(C)]
pub struct NucleoHandle {
    internal: Nucleo<(u32, Box<[u8]>)>,
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
    pub index: u32,
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
pub struct NucleoDartMatch {
    pub score: u32,
    pub index: u32,
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
pub struct NucleoDartString {
    pub index: u32,
    pub ptr: *const u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_add(ptr: *mut NucleoHandle, item: NucleoDartStringMut) {
    let reference: &mut NucleoHandle = unsafe { ptr.as_mut().unwrap() };
    let injector = reference.internal.injector();

    let slice_ref = unsafe { std::slice::from_raw_parts_mut(item.ptr, item.len) };
    let slice_boxed: Box<[u8]> = slice_ref.into();

    injector.push((item.index, slice_boxed), |v, z| {
        let boxed_str: Box<str> = match String::from_utf8(v.1.clone().into_vec()) {
            Ok(v) => v.into_boxed_str(),
            Err(err) => {
                println!("ERR {}", err);
                return;
            }
        };
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

        injector.push((item.index, slice_boxed), |v, z| {
            let boxed_str: Box<str> = match String::from_utf8(v.1.clone().into_vec()) {
                Ok(v) => v.into_boxed_str(),
                Err(err) => {
                    println!("ERR {}", err);
                    return;
                }
            };
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

type SnapshotHandle = nucleo::Snapshot<(u32, Box<[u8]>)>;

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
        index: item.data.0,
        ptr: item.data.1.as_ptr(),
        len: item.data.1.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_matched_item(
    handle: *const SnapshotHandle,
    index: u32,
) -> NucleoDartMatch {
    let reference = unsafe { handle.as_ref().unwrap() };
    let item = reference.get_matched_item(index).unwrap();
    NucleoDartMatch {
        score: reference.matches()[index as usize].score,
        index: item.data.0,
        ptr: item.data.1.as_ptr(),
        len: item.data.1.len(),
    }
}

pub type AppendCallbackFn = extern "C" fn(NucleoDartMatch);

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_snapshot_get_matched_items(
    handle: *const SnapshotHandle,
    start: u32,
    end: u32,
    cb: AppendCallbackFn,
) {
    let reference = unsafe { handle.as_ref().unwrap() };
    for i in start..end {
        let item = reference.get_matched_item(i).unwrap();
        cb(NucleoDartMatch {
            score: reference.matches()[i as usize].score,
            index: item.data.0,
            ptr: item.data.1.as_ptr(),
            len: item.data.1.len(),
        })
    }
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct NucleoDartMMatch {
    pub score: u32,
    pub idx: u32,
}

#[repr(C)]
pub struct NucleoDartSnapshot2Match {
    pub mtch: NucleoDartMMatch,
    pub handle: *const SnapshotHandle,
}

#[repr(C)]
pub struct NucleoDartSnapshot2 {
    pub matches: *const NucleoDartSnapshot2Match,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_destroy_join(handle: *const NucleoDartSnapshot2) {
    let boxed = Box::new(unsafe { std::slice::from_raw_parts((*handle).matches, (*handle).len) });
    drop(boxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn nucleo_dart_join_snapshot(
    handle_a: *const SnapshotHandle,
    handle_b: *const SnapshotHandle,
) -> NucleoDartSnapshot2 {
    let reference_a = unsafe { handle_a.as_ref().unwrap() };
    let reference_b = unsafe { handle_b.as_ref().unwrap() };
    let matches_a = reference_a.matches();
    let matches_b = reference_b.matches();

    let mut matches_result: Vec<NucleoDartSnapshot2Match> = vec![];
    let mut previous_seen_matches = std::collections::HashMap::<&u32, (usize, i32, u32)>::new();

    for (i, mtch) in matches_a.iter().enumerate() {
        let item = unsafe { reference_a.get_item_unchecked(mtch.idx) };
        let previous_score = previous_seen_matches.get(&item.data.0);
        match &previous_score {
            Some(_) => panic!("invalid state matches_a should not have repeated indxs"),
            None => {
                matches_result.push(NucleoDartSnapshot2Match {
                    mtch: NucleoDartMMatch {
                        idx: i as u32,
                        score: mtch.score,
                    },
                    handle: handle_a,
                });
                previous_seen_matches
                    .insert(&item.data.0, (matches_result.len() - 1, 0, mtch.score));
            }
        }
    }

    for (i, mtch) in matches_b.iter().enumerate() {
        let item = unsafe { reference_a.get_item_unchecked(mtch.idx) };
        let previous_score = previous_seen_matches.get(&item.data.0);
        match &previous_score {
            Some(v) => {
                if mtch.score > v.2 {
                    matches_result[v.0] = NucleoDartSnapshot2Match {
                        mtch: NucleoDartMMatch {
                            score: mtch.score,
                            idx: i as u32,
                        },
                        handle: handle_b,
                    };
                    previous_seen_matches.insert(&item.data.0, (v.0, 0, mtch.score));
                }
            }
            None => {
                matches_result.push(NucleoDartSnapshot2Match {
                    mtch: NucleoDartMMatch {
                        score: mtch.score,
                        idx: i as u32,
                    },
                    handle: handle_b,
                });
                previous_seen_matches
                    .insert(&item.data.0, (matches_result.len() - 1, 0, mtch.score));
            }
        };
    }
    let boxed = matches_result.into_boxed_slice();
    let response = NucleoDartSnapshot2 {
        matches: boxed.as_ptr(),
        len: boxed.len(),
    };
    Box::leak(boxed);
    return response;
}
