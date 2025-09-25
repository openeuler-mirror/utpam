/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::unwrap_or_default, clippy::not_unsafe_ptr_arg_deref)]
use crate::pam_conv::pamconv_to_utpamconv;
use crate::pam_conv::PamConv;
use crate::pam_private::pam_handle_t;
use libc::{c_char, c_int};
use std::ffi::CStr;
use std::rc::Rc;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_start::utpam_start;
use utpam::utpam_syslog::*;

// C兼容的初始化函数
#[no_mangle]
pub extern "C" fn pam_start(
    service_name: *const c_char,
    user: *const c_char,
    pam_conversation: *const PamConv,
    pamh: *mut *mut pam_handle_t,
) -> c_int {
    //初始化日志
    log_init();

    if pamh.is_null() {
        log::debug!(
            "{} {}",
            LOG_ERR,
            "pam_start: invalid argument: pamh == NULL"
        );
        return PAM_SYSTEM_ERR as c_int;
    }
    let service_name = if service_name.is_null() {
        log::debug!(
            "{} {}",
            LOG_ERR,
            "pam_start: invalid argument: service == NULL"
        );
        return PAM_SYSTEM_ERR as c_int;
    } else {
        unsafe { CStr::from_ptr(service_name).to_string_lossy().into_owned() }
    };
    if pam_conversation.is_null() {
        log::debug!(
            "{} {}",
            LOG_ERR,
            "pam_start: invalid argument: conv == NULL"
        );
        return PAM_SYSTEM_ERR as c_int;
    }

    let user = if user.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(user).to_string_lossy().into_owned() })
    };
    let conv = pamconv_to_utpamconv(pam_conversation);
    let mut utpamh: Option<Box<UtpamHandle>> = None;
    let result = utpam_start(service_name, user, Some(Rc::new(conv)), &mut utpamh);

    if let Some(handle) = utpamh {
        unsafe {
            // 将Rust的UtpamHandle包装成pam_handle_t
            let pamh_struct = pam_handle_t {
                data: Box::into_raw(handle),
            };
            *pamh = Box::into_raw(Box::new(pamh_struct));
        }
    }

    result as c_int
}
