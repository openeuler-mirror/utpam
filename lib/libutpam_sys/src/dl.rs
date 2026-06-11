/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use libloading::{Library, Symbol};
use std::path::Path;

pub fn open_library(path: impl AsRef<Path>) -> Result<Library, libloading::Error> {
    unsafe { Library::new(path.as_ref()) }
}

pub fn get_symbol<'a, T>(
    lib: &'a Library,
    symbol: &[u8],
) -> Result<Symbol<'a, T>, libloading::Error> {
    unsafe { lib.get::<T>(symbol) }
}
