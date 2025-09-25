/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut, unused_variables)]
///存放utpam公共的结构体和常量
use std::any::Any;
use std::rc::Rc;

pub const PAM_PROMPT_ECHO_OFF: u8 = 1;
pub const PAM_BINARY_PROMPT: u8 = 7;
pub const PAM_ERROR_MSG: u8 = 3;
pub const PAM_TEXT_INFO: u8 = 4;

pub const PAM_SUCCESS: u8 = 0;
pub const PAM_OPEN_ERR: u8 = 1;
pub const PAM_SYMBOL_ERR: u8 = 2;
pub const PAM_SERVICE_ERR: u8 = 3;
pub const PAM_SYSTEM_ERR: u8 = 4;
pub const PAM_BUF_ERR: u8 = 5;
pub const PAM_PERM_DENIED: u8 = 6;
pub const PAM_AUTH_ERR: u8 = 7;
pub const PAM_CRED_INSUFFICIENT: u8 = 8;
pub const PAM_AUTHINFO_UNAVAIL: u8 = 9;
pub const PAM_USER_UNKNOWN: u8 = 10;
pub const PAM_MAXTRIES: u8 = 11;
pub const PAM_NEW_AUTHTOK_REQD: u8 = 12;
pub const PAM_ACCT_EXPIRED: u8 = 13;
pub const PAM_SESSION_ERR: u8 = 14;
pub const PAM_CRED_UNAVAIL: u8 = 15;
pub const PAM_CRED_EXPIRED: u8 = 16;
pub const PAM_CRED_ERR: u8 = 17;
pub const PAM_NO_MODULE_DATA: u8 = 18;
pub const PAM_CONV_ERR: u8 = 19;
pub const PAM_AUTHTOK_ERR: u8 = 20;
pub const PAM_AUTHTOK_RECOVERY_ERR: u8 = 21;
pub const PAM_AUTHTOK_LOCK_BUSY: u8 = 22;
pub const PAM_AUTHTOK_DISABLE_AGING: u8 = 23;
pub const PAM_TRY_AGAIN: u8 = 24;
pub const PAM_IGNORE: u8 = 25;
pub const PAM_ABORT: u8 = 26;
pub const PAM_AUTHTOK_EXPIRED: u8 = 27;
pub const PAM_MODULE_UNKNOWN: u8 = 28;
pub const PAM_BAD_ITEM: u8 = 29;
pub const PAM_CONV_AGAIN: u8 = 30;
pub const PAM_INCOMPLETE: u8 = 31;
pub const PAM_RETURN_VALUES: usize = 32;

pub const LOG_EMERG: u8 = 0;
pub const LOG_ALERT: u8 = 1;
pub const LOG_CRIT: u8 = 2;
pub const LOG_ERR: u8 = 3;
pub const LOG_WARNING: u8 = 4;
pub const LOG_NOTICE: u8 = 5;
pub const LOG_INFO: u8 = 6;
pub const LOG_DEBUG: u8 = 7;

pub const PAM_SERVICE: i8 = 1;
pub const PAM_USER: i8 = 2;
pub const PAM_TTY: i8 = 3;
pub const PAM_RHOST: i8 = 4;
pub const PAM_CONV: i8 = 5;
pub const PAM_AUTHTOK: i8 = 6;
pub const PAM_OLDAUTHTOK: i8 = 7;
pub const PAM_RUSER: i8 = 8;
pub const PAM_USER_PROMPT: i8 = 9;
pub const PAM_FAIL_DELAY: i8 = 10;
pub const PAM_XDISPLAY: i8 = 11;
pub const PAM_XAUTHDATA: i8 = 12;
pub const PAM_AUTHTOK_TYPE: i8 = 13;

pub const PAM_ESTABLISH_CRED: u32 = 0x0002;

pub const PAM_PRELIM_CHECK: u32 = 0x4000;
pub const PAM_UPDATE_AUTHTOK: u32 = 0x2000;

pub const PAM_DATA_REPLACE: i32 = 0x20000000;

pub const PAM_SILENT: u32 = 0x8000;

pub const PAM_MAX_MSG_SIZE: usize = 512;

pub const PAM_PROMPT_ECHO_ON: u8 = 2;

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

/// 安全地覆盖字符串或字节数组
#[macro_export]
macro_rules! utpam_overwrite_string {
    ($x:expr) => {
        if !$x.is_empty() {
            $x.zeroize();
        }
    };
}

type MiscConv =
    Box<dyn Fn(usize, &[UtpamMessage], &mut Option<Vec<UtpamResponse>>, Option<Rc<dyn Any>>) -> u8>;

pub struct UtpamConv {
    pub conv: Option<MiscConv>,
    pub appdata_ptr: Option<Rc<dyn Any>>,
}
pub struct UtpamResponse {
    pub resp: Vec<String>,
    pub resp_retcode: isize,
}

pub struct UtpamMessage {
    pub msg_style: u8,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UtpamXAuthData {
    pub namelen: i32,
    pub name: Option<String>,
    pub datalen: i32,
    pub data: Vec<String>,
}

impl UtpamXAuthData {
    /// 清空 UtpamXAuthData 结构体的所有字段
    pub fn clear(&mut self) {
        self.namelen = 0;
        self.name = None;
        self.datalen = 0;
        self.data.clear();
    }
}
