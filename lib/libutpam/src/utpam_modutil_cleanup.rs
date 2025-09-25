/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::utpam::UtpamHandle;
use std::any::Any;
use std::rc::Rc;
// 释放内存
pub fn utpam_modutil_cleanup(
    _pamh: &mut UtpamHandle,
    data: Option<Rc<dyn Any>>,
    _error_status: i32,
) {
    if let Some(data) = data {
        drop(data);
    }
}
