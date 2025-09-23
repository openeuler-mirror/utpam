/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::PAM_SYSTEM_ERR;
use crate::utpam::{UtpamHandle, PAM_ACCOUNT, PAM_CALLED_FROM_MODULE};
use crate::utpam_dispatch::utpam_dispatch;
use crate::{IF_NO_UTPAMH, UTPAM_FROM_MODULE};
pub fn utpam_acct_mgmt(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> u8 {
    ////检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    //模块调度
    utpam_dispatch(utpamh, flags, PAM_ACCOUNT)
}
