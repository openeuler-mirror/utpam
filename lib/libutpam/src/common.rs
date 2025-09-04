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

pub const PAM_SESSION_ERR: i32 = 7;
pub const PAM_ABORT: i32 = 26;
pub const PAM_INCOMPLETE: i32 = 31;

pub const PAM_ESTABLISH_CRED: u32 = 0x0002;

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
