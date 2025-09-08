/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut, unused_variables)]
///存放utpam公共的结构体和常量
use std::any::Any;

pub const PAM_SUCCESS: i32 = 0;
pub const PAM_SYSTEM_ERR: i32 = 4;
pub const PAM_BUF_ERR: i32 = 5;
pub const PAM_NEW_AUTHTOK_REQD: i32 = 12;
pub const PAM_IGNORE: i32 = 25;

pub const PAM_SESSION_ERR: i32 = 7;
pub const PAM_ABORT: i32 = 26;
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
