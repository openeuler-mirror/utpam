/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::*;
use utpam::utpam::*;

//以下6个是SPI函数，函数名、入参和返回值不能变，这是固定的，内容可以变

//用户认证
pub fn utpam_sm_authenticate(
    mut _utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}

//设置凭证
pub fn utpam_sm_setcred(
    mut _utpam: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}

//账户管理
pub fn utpam_sm_acct_mgmt(
    mut _utpam: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}

//密码管理
pub fn utpam_sm_chauthtok(
    mut _utpam: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}

//打开会话
pub fn utpam_sm_open_session(
    mut _utpam: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}

//关闭会话
pub fn utpam_sm_close_session(
    mut _utpam: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SESSION_ERR
}
