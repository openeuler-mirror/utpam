/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(
    unused_variables,
    dead_code,
    unreachable_code,
    non_snake_case,
    unreachable_patterns,
    clippy::needless_return
)]

use crate::common::*;
use crate::utpam::UtpamHandle;
//未补充完，需要调用国际化函数tgettext();
#[no_mangle]
pub fn pam_strerror(utpamh: &mut Box<UtpamHandle>, errnum: u8) -> &'static str {
    match errnum {
        PAM_SUCCESS => return "Success",
        PAM_ABORT => return "Critical error - immediate abort",
        PAM_OPEN_ERR => return "Failed to load module",
        PAM_SYMBOL_ERR => return "Symbol not found",
        PAM_SERVICE_ERR => return "Error in service module",
        PAM_SYSTEM_ERR => return "System error",
        PAM_BUF_ERR => return "Memory buffer error",
        PAM_PERM_DENIED => return "Permission denied",
        PAM_AUTH_ERR => return "Authentication failure",
        PAM_CRED_INSUFFICIENT => return "Insufficient credentials to access authentication data",
        PAM_AUTHINFO_UNAVAIL => {
            return "Authentication service cannot retrieve authentication info"
        }
        PAM_USER_UNKNOWN => return "User not known to the underlying authentication module",
        PAM_MAXTRIES => return "Have exhausted maximum number of retries for service",
        PAM_NEW_AUTHTOK_REQD => return "Authentication token is no longer valid; new one required",
        PAM_ACCT_EXPIRED => return "User account has expired",
        PAM_SESSION_ERR => return "Cannot make/remove an entry for the specified session",
        PAM_CRED_UNAVAIL => return "Authentication service cannot retrieve user credentials",
        PAM_CRED_EXPIRED => return "User credentials expired",
        PAM_CRED_ERR => return "Failure setting user credentials",
        PAM_NO_MODULE_DATA => return "No module specific data is present",
        PAM_BAD_ITEM => return "Bad item passed to pam_*_item()",
        PAM_CONV_ERR => return "Conversation error",
        PAM_AUTHTOK_ERR => return "Authentication token manipulation error",
        PAM_AUTHTOK_RECOVERY_ERR => return "Authentication information cannot be recovered",
        PAM_AUTHTOK_LOCK_BUSY => return "Authentication token lock busy",
        PAM_AUTHTOK_DISABLE_AGING => return "Authentication token aging disabled",
        PAM_TRY_AGAIN => return "Failed preliminary check by password service",
        PAM_IGNORE => return "The return value should be ignored by PAM dispatch",
        PAM_MODULE_UNKNOWN => return "Module is unknown",
        PAM_AUTHTOK_EXPIRED => return "Authentication token expired",
        PAM_CONV_AGAIN => return "Conversation is waiting for event",
        PAM_INCOMPLETE => return "Application needs to call libpam again",
        _ => return "Unknown PAM error",
    }
}
