/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_variables)]
use crate::common::*;
use crate::utpam::UtpamHandle;
pub fn utpam_end(utpamh: &mut Option<Box<UtpamHandle>>, pam_status: i32) -> i32 {
    //待开发
    PAM_SUCCESS
}
