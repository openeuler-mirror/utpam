/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::utpam::UtpamHandle;
// 不透明的结构体指针
#[repr(C)]
pub struct pam_handle_t {
    pub data: *mut UtpamHandle,
}
