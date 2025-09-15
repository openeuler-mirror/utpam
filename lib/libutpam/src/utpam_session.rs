/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::PAM_SYSTEM_ERR;
use crate::utpam::{UtpamHandle, PAM_CALLED_FROM_MODULE, PAM_CLOSE_SESSION, PAM_OPEN_SESSION};
use crate::utpam_dispatch::utpam_dispatch;
use crate::{IF_NO_UTPAMH, UTPAM_FROM_MODULE};

//打开会话
pub fn utpam_open_session(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    ////检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    //模块调度
    utpam_dispatch(utpamh, flags, PAM_OPEN_SESSION)
}

//注销或结束会话
pub fn utpam_close_session(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    ////检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    //模块调度
    utpam_dispatch(utpamh, flags, PAM_CLOSE_SESSION)
}
