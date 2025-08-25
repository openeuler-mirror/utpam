/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]

pub struct UtpamEnviron {
    entries: usize,
    requested: usize,
    list: Vec<String>,
}

impl UtpamEnviron {
    pub fn new() -> Self {
        UtpamEnviron {
            entries: 0,
            requested: 0,
            list: Vec::new(),
        }
    }
}

impl Default for UtpamEnviron {
    fn default() -> Self {
        Self::new()
    }
}
