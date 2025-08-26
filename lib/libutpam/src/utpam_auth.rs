/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_variables)]
use crate::common::*;
use crate::utpam::*;
use crate::utpam_dispatch::utpam_dispatch;

pub fn utpam_authenticate(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    let retval = utpam_dispatch(utpamh, flags, PAM_AUTHENTICATE);
    if retval != PAM_INCOMPLETE {
        //重置认证相关状态
        //等待延迟
    }

    retval
}

pub fn utpam_setcred(utpamh: &mut Option<Box<UtpamHandle>>, mut flags: u32) -> i32 {
    if flags != 0 {
        flags = PAM_ESTABLISH_CRED;
    }

    utpam_dispatch(utpamh, flags, PAM_AUTHENTICATE)

    //日志处理
}
