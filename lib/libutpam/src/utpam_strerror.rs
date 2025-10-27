/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::*;

#[no_mangle]
pub fn pam_strerror(errnum: u8) -> &'static str {
    match errnum {
        PAM_SUCCESS => "Success",
        PAM_ABORT => "Critical error - immediate abort",
        PAM_OPEN_ERR => "Failed to load module",
        PAM_SYMBOL_ERR => "Symbol not found",
        PAM_SERVICE_ERR => "Error in service module",
        PAM_SYSTEM_ERR => "System error",
        PAM_BUF_ERR => "Memory buffer error",
        PAM_PERM_DENIED => "Permission denied",
        PAM_AUTH_ERR => "Authentication failure",
        PAM_CRED_INSUFFICIENT => "Insufficient credentials to access authentication data",
        PAM_AUTHINFO_UNAVAIL => "Authentication service cannot retrieve authentication info",
        PAM_USER_UNKNOWN => "User not known to the underlying authentication module",
        PAM_MAXTRIES => "Have exhausted maximum number of retries for service",
        PAM_NEW_AUTHTOK_REQD => "Authentication token is no longer valid; new one required",
        PAM_ACCT_EXPIRED => "User account has expired",
        PAM_SESSION_ERR => "Cannot make/remove an entry for the specified session",
        PAM_CRED_UNAVAIL => "Authentication service cannot retrieve user credentials",
        PAM_CRED_EXPIRED => "User credentials expired",
        PAM_CRED_ERR => "Failure setting user credentials",
        PAM_NO_MODULE_DATA => "No module specific data is present",
        PAM_BAD_ITEM => "Bad item passed to pam_*_item()",
        PAM_CONV_ERR => "Conversation error",
        PAM_AUTHTOK_ERR => "Authentication token manipulation error",
        PAM_AUTHTOK_RECOVERY_ERR => "Authentication information cannot be recovered",
        PAM_AUTHTOK_LOCK_BUSY => "Authentication token lock busy",
        PAM_AUTHTOK_DISABLE_AGING => "Authentication token aging disabled",
        PAM_TRY_AGAIN => "Failed preliminary check by password service",
        PAM_IGNORE => "The  value should be ignored by PAM dispatch",
        PAM_MODULE_UNKNOWN => "Module is unknown",
        PAM_AUTHTOK_EXPIRED => "Authentication token expired",
        PAM_CONV_AGAIN => "Conversation is waiting for event",
        PAM_INCOMPLETE => "Application needs to call libpam again",
        _ => "Unknown PAM error",
    }
}
