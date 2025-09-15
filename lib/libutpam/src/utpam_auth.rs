/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_variables)]
use crate::common::*;
use crate::utpam::*;
use crate::utpam_delay::utpam_await_timer;
use crate::utpam_dispatch::utpam_dispatch;
use crate::utpam_misc::utpam_sanitize;
use crate::{IF_NO_UTPAMH, UTPAM_FROM_MODULE};

pub fn utpam_authenticate(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    ////检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    if utpamh.former.choice == PAM_NOT_STACKED {
        utpam_sanitize(utpamh);
        utpamh.fail_delay.utpam_start_timer();
    }

    //模块调度
    let retval = utpam_dispatch(utpamh, flags, PAM_AUTHENTICATE);

    if retval != PAM_INCOMPLETE {
        utpam_sanitize(utpamh);
        utpam_await_timer(utpamh, retval);
    } else {
        println!("will resume when ready"); //后续改为日记记录
    }

    retval
}

pub fn utpam_setcred(utpamh: &mut Option<Box<UtpamHandle>>, mut flags: u32) -> i32 {
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if flags != 0 {
        flags = PAM_ESTABLISH_CRED;
    }

    utpam_dispatch(utpamh, flags, PAM_AUTHENTICATE)

    //日志处理
}
