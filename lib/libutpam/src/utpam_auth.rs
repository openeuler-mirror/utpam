/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_variables)]
use crate::common::*;
use crate::utpam::UtpamHandle;

pub fn utpam_authenticate(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    //待开发
    PAM_SUCCESS
}

pub fn utpam_setcred(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    //待开发
    PAM_SUCCESS
}
