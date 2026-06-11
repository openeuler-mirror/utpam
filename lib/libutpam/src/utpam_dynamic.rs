/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use crate::utpam::CallSpi;
use libloading::{Library, Symbol};
use utpam_sys::dl;

//加载动态库
pub fn utpam_dlopen(path: String) -> Result<Library, Box<dyn std::error::Error>> {
    Ok(dl::open_library(path)?)
}

//从由 handle 指定的已加载库中查找名为 symbol 的符号
pub fn utpam_dlsym<'a>(
    handle: &'a Option<&Library>,
    symbol: &'a [u8],
) -> Result<Symbol<'a, CallSpi>, Box<dyn std::error::Error>> {
    if let Some(h) = handle {
        Ok(dl::get_symbol::<CallSpi>(h, symbol)?)
    } else {
        Err("Error: handle is None".into())
    }
}
