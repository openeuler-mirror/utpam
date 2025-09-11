/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut, unused_variables)]
///存放utpam公共的结构体和常量
use std::any::Any;

pub const PAM_SUCCESS: i32 = 0;
pub const PAM_OPEN_ERR: i32 = 1;
pub const PAM_SYMBOL_ERR: i32 = 2;
pub const PAM_SERVICE_ERR: i32 = 3;
pub const PAM_SYSTEM_ERR: i32 = 4;
pub const PAM_BUF_ERR: i32 = 5;
pub const PAM_PERM_DENIED: i32 = 6;
pub const PAM_AUTH_ERR: i32 = 7;
pub const PAM_CRED_INSUFFICIENT: i32 = 8;
pub const PAM_AUTHINFO_UNAVAIL: i32 = 9;
pub const PAM_USER_UNKNOWN: i32 = 10;
pub const PAM_MAXTRIES: i32 = 11;
pub const PAM_NEW_AUTHTOK_REQD: i32 = 12;
pub const PAM_ACCT_EXPIRED: i32 = 13;
pub const PAM_SESSION_ERR: i32 = 14;
pub const PAM_CRED_UNAVAIL: i32 = 15;
pub const PAM_CRED_EXPIRED: i32 = 16;
pub const PAM_CRED_ERR: i32 = 17;
pub const PAM_NO_MODULE_DATA: i32 = 18;
pub const PAM_CONV_ERR: i32 = 19;
pub const PAM_AUTHTOK_ERR: i32 = 20;
pub const PAM_AUTHTOK_RECOVERY_ERR: i32 = 21;
pub const PAM_AUTHTOK_LOCK_BUSY: i32 = 22;
pub const PAM_AUTHTOK_DISABLE_AGING: i32 = 23;
pub const PAM_TRY_AGAIN: i32 = 24;
pub const PAM_IGNORE: i32 = 25;
pub const PAM_ABORT: i32 = 26;
pub const PAM_AUTHTOK_EXPIRED: i32 = 27;
pub const PAM_MODULE_UNKNOWN: i32 = 28;
pub const PAM_BAD_ITEM: i32 = 29;
pub const PAM_CONV_AGAIN: i32 = 30;
pub const PAM_INCOMPLETE: i32 = 31;
pub const PAM_RETURN_VALUES: usize = 32;

pub const PAM_ESTABLISH_CRED: u32 = 0x0002;

pub const PAM_TOKEN_RETURNS: [&str; 33] = [
    "success",
    "open_err",
    "symbol_err",
    "service_err",
    "system_err",
    "buf_err",
    "perm_denied",
    "auth_err",
    "cred_insufficient",
    "authinfo_unavail",
    "user_unknown",
    "maxtries",
    "new_authtok_reqd",
    "acct_expired",
    "session_err",
    "cred_unavail",
    "cred_expired",
    "cred_err",
    "no_module_data",
    "conv_err",
    "authtok_err",
    "authtok_recover_err",
    "authtok_lock_busy",
    "authtok_disable_aging",
    "try_again",
    "ignore",
    "abort",
    "authtok_expired",
    "module_unknown",
    "bad_item",
    "conv_again",
    "incomplete",
    "default",
];

pub const PAM_TOKEN_ACTIONS: [&str; 6] = [
    "ignore", /*  0 */
    "ok",     /* -1 */
    "done",   /* -2 */
    "bad",    /* -3 */
    "die",    /* -4 */
    "reset",  /* -5 */
];

pub type MiscConv = fn(
    num_msg: isize,
    msg: &[UtpamMessage],
    resp: &mut Option<Vec<UtpamResponse>>,
    appdata_ptr: Option<Box<dyn Any>>,
) -> isize;

#[derive(Debug)]
pub struct UtpamConv {
    pub conv: MiscConv,
    pub appdata_ptr: Option<Box<dyn Any>>,
}
pub struct UtpamResponse {
    pub resp: Vec<String>,
    pub resp_retcode: isize,
}

pub struct UtpamMessage {
    pub msg_style: isize,
    pub msg: Vec<String>,
}

#[derive(Debug)]
pub struct UtpamXAuthData {
    pub namelen: usize,
    pub name: Option<String>,
    pub datalen: usize,
    pub data: Vec<u8>,
}

#[macro_export]
macro_rules! IF_NO_UTPAMH {
    ($expr:expr, $err:expr) => {{
        match $expr {
            Some(ref mut value) => value,
            None => return $err,
        }
    }};
}
