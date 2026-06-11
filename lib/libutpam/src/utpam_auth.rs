/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use crate::common::*;
use crate::utpam::*;
use crate::utpam_delay::utpam_await_timer;
use crate::utpam_dispatch::utpam_dispatch;
use crate::utpam_misc::utpam_sanitize;
use crate::{D, IF_NO_UTPAMH, UTPAM_FROM_MODULE};

/// 身份验证
pub fn utpam_authenticate(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> u8 {
    D!("called");

    //检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        D!("called from module!?");
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
        D!("exiting");
    } else {
        D!("will resume when ready");
    }

    retval
}

/// 设置用户凭证
pub fn utpam_setcred(utpamh: &mut Option<Box<UtpamHandle>>, mut flags: u32) -> u8 {
    D!("called");

    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        D!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    if flags == 0 {
        flags = PAM_ESTABLISH_CRED;
    }

    let retval = utpam_dispatch(utpamh, flags, PAM_SETCRED);

    D!("exiting");

    retval
}
