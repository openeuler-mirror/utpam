/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///存放utpam公共的结构体和常量

pub const PAM_SUCCESS: i32 = 0;
pub const PAM_SESSION_ERR: i32 = 7;

pub trait Conv {
    fn conv(
        &self,
        num_msg: isize,
        msg: &[UtpamMessage],
        resp: &mut Option<Vec<UtpamResponse>>,
        appdata_ptr: Vec<String>,
    ) -> isize;
}
pub struct UtpamConv {
    pub conv: Option<Box<dyn Conv>>,
    pub appdata_ptr: Vec<String>,
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
