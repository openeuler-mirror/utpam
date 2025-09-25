/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::unwrap_or_default, clippy::not_unsafe_ptr_arg_deref)]

use crate::pam_private::pam_handle_t;
use libc::c_int;
use utpam::utpam::UtpamHandle;
use utpam::utpam_account::utpam_acct_mgmt;

// C兼容的账号管理函数
#[no_mangle]
pub extern "C" fn pam_acct_mgmt(pamh: *mut pam_handle_t, flags: c_int) -> c_int {
    unsafe {
        if pamh.is_null() || (*pamh).data.is_null() {
            return -1; // 返回错误码表示无效的pam_handle_t
        }
    }

    let utpamh = unsafe { &mut ((*pamh).data as *mut Option<Box<UtpamHandle>>).as_mut() };

    if let Some(handle) = utpamh {
        utpam_acct_mgmt(handle, flags as u32) as c_int
    } else {
        println!("utpamh is null, flags: {}", flags);
        0
    }
}
