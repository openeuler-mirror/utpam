/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::*;
use crate::utpam::*;
use crate::utpam_data::utpam_free_data;
use crate::utpam_env::utpam_drop_env;
use crate::{utpam_overwrite_string, IF_NO_UTPAMH, UTPAM_FROM_MODULE};

use zeroize::Zeroize;

//utpam_overwrite_string宏用来清理字符串
pub fn utpam_end(utpamh: &mut Option<Box<UtpamHandle>>, pam_status: i32) -> i32 {
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from application!?");
        return PAM_SYSTEM_ERR;
    }
    utpam_free_data(utpamh, pam_status);

    utpam_drop_env(utpamh);

    utpam_overwrite_string!(utpamh.authtok);
    utpam_overwrite_string!(utpamh.oldauthtok);

    if let Some(ref mut prompt) = utpamh.former.prompt {
        utpam_overwrite_string!(prompt);
    }
    utpam_overwrite_string!(utpamh.service_name);
    utpam_overwrite_string!(utpamh.user);

    if let Some(ref mut confdir) = utpamh.confdir.to_str() {
        utpam_overwrite_string!(confdir.to_string());
    }

    utpam_overwrite_string!(utpamh.prompt);
    utpam_overwrite_string!(utpamh.tty);
    utpam_overwrite_string!(utpamh.rhost);
    utpam_overwrite_string!(utpamh.ruser);

    utpamh.fail_delay.delay_fn_ptr = None;

    utpam_overwrite_string!(utpamh.xdisplay);

    if let Some(ref mut name) = utpamh.xauth.name {
        utpam_overwrite_string!(name);
    }

    utpam_overwrite_string!(utpamh.authtok_type);

    PAM_SUCCESS
}
