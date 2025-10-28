/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use users::get_current_uid;
use utpam::common::*;
use utpam::utpam::*;

const PAM_DEBUG_ARG: u8 = 1;

fn _pam_parse(
    _utpamh: &mut Option<Box<UtpamHandle>>,
    _argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    let mut ctrl = 0;

    if let Some(ref args) = argv {
        for arg in args.iter() {
            if arg == "debug" {
                ctrl |= PAM_DEBUG_ARG;
            } else {
                // pam_syslog!(utpamh, LOG_ERR, "unknown option: {}", arg);
                println!("unknown option: {}", arg);
            }
        }
    }
    ctrl
}

fn check_for_root(mut _utpamh: &mut Option<Box<UtpamHandle>>, ctrl: u8) -> u8 {
    let mut retval = PAM_AUTH_ERR;
    if get_current_uid() == 0 {
        retval = PAM_SUCCESS;
    }

    let result = {
        if retval == PAM_SUCCESS {
            "success"
        } else {
            "failed"
        }
    };

    if (ctrl & PAM_DEBUG_ARG) != 0 {
        // pam_syslog!(utpamh, LOG_DEBUG, "root check {}", result);
        println!("root check {}", result);
    }

    retval
}

//用户认证
#[no_mangle]
pub fn utpam_sm_authenticate(
    utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    let ctrl = _pam_parse(utpamh, argc, argv);

    check_for_root(utpamh, ctrl)
}

//设置凭证
#[no_mangle]
pub fn utpam_sm_setcred(
    _utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_SUCCESS
}

//账户管理
#[no_mangle]
pub fn utpam_sm_acct_mgmt(
    utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    let ctrl = _pam_parse(utpamh, argc, argv);

    check_for_root(utpamh, ctrl)
}

//密码管理
#[no_mangle]
pub fn utpam_sm_chauthtok(
    utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    let ctrl = _pam_parse(utpamh, argc, argv);

    check_for_root(utpamh, ctrl)
}
