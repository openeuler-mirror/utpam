/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;
use utpam::utpam_strerror::utpam_strerror;

// 使用 Mutex 来确保线程安全
lazy_static::lazy_static! {
    static ref STRINGS: Mutex<Vec<CString>> = Mutex::new(Vec::new());
}

#[no_mangle]
pub extern "C" fn pam_strerror(errnum: i32) -> *const c_char {
    // 调用 utpam_strerror 并将其转换为 C 兼容的字符串
    let rust_str = utpam_strerror(errnum as u8);

    // 将 Rust 字符串转换为 C 兼容的字符串
    let c_string = CString::new(rust_str).expect("CString::new failed");

    // 将 C 兼容的字符串存储在静态变量中
    let mut strings = STRINGS.lock().unwrap();
    let ptr = c_string.as_ptr();
    strings.push(c_string); // 确保字符串不会被释放

    ptr
}

