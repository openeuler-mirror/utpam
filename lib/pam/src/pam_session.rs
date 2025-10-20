/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::unwrap_or_default, clippy::not_unsafe_ptr_arg_deref)]

use crate::pam_private::pam_handle_t;
use libc::c_int;
use utpam::common::PAM_SYSTEM_ERR;
use utpam::utpam::UtpamHandle;
use utpam::utpam_session::{utpam_close_session, utpam_open_session};

// C兼容的开启会话函数
#[no_mangle]
pub extern "C" fn pam_open_session(pamh: pam_handle_t, flags: c_int) -> c_int {
    if pamh.data.is_null() {
        return PAM_SYSTEM_ERR as c_int;
    }

    let utpamh: &mut Option<Box<UtpamHandle>> =
        unsafe { &mut *(pamh.data as *mut Option<Box<UtpamHandle>>) };

    utpam_open_session(utpamh, flags as u32) as c_int
}

// C兼容的关闭会话函数
#[no_mangle]
pub extern "C" fn pam_close_session(pamh: pam_handle_t, flags: c_int) -> c_int {
    if pamh.data.is_null() {
        return PAM_SYSTEM_ERR as c_int;
    }

    let utpamh: &mut Option<Box<UtpamHandle>> =
        unsafe { &mut *(pamh.data as *mut Option<Box<UtpamHandle>>) };

    utpam_close_session(utpamh, flags as u32) as c_int
}
