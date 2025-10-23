/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]
use crate::common::*;
use crate::pam_syslog;
use crate::utpam::*;
use crate::utpam_syslog::*;

use std::io::Error;

fn _pam_audit_open(utpamh: &UtpamHandle) -> i32 {
    // rust暂未相关第三方库,暂不重构
    // let audit_fd = audit_open(utpamh);

    let audit_fd = 0;
    if audit_fd < 0 {
        let last_error = Error::last_os_error();
        pam_syslog!(utpamh, LOG_CRIT, "audit_open() failed: {}", last_error);
        return -1;
    }
    //  let errno = Error::last_os_error();
    //  match errno.kind(){
    //  }

    // EINVAL 参数错误
    // EPROTONOSUPPORT 协议不支持
    // EAFNOSUPPORT 地址族不支持
    //  if errno == EINVAL || errno == EPROTONOSUPPORT || errno == EAFNOSUPPORT {
    //     return -2;
    //  }

    audit_fd
}

fn pam_modutil_audit_write(utpamh: &UtpamHandle, _type: i32, _message: &str, _retval: i32) -> i32 {
    #[cfg(feature = "have_audit")]
    {
        let audit_fd = _pam_audit_open(utpamh);
        if audit_fd == -1 {
            return PAM_SYSTEM_ERR as i32;
        } else if audit_fd == -2 {
            return _retval;
        }

        audit_fd
    }

    #[cfg(not(feature = "have_audit"))]
    {
        return PAM_SUCCESS as i32;
    }
}
