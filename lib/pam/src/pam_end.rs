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
use utpam::utpam_end::utpam_end;

// C兼容的结束会话函数
#[no_mangle]
pub extern "C" fn pam_end(pamh: pam_handle_t, pam_status: c_int) -> c_int {
    if pamh.data.is_null() {
        return PAM_SYSTEM_ERR as c_int;
    }

    let utpamh: &mut Option<Box<UtpamHandle>> =
        unsafe { &mut *(pamh.data as *mut Option<Box<UtpamHandle>>) };

    utpam_end(utpamh, pam_status) as c_int
}
