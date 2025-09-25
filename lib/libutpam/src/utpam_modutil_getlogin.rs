/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::*;
use crate::utpam::*;
use crate::utpam_data::{utpam_get_data, utpam_set_data};
use crate::utpam_modutil_cleanup::utpam_modutil_cleanup;
use std::rc::Rc;
use users::get_current_username;

const _PAMMODUTIL_GETLOGIN: &str = "_pammodutil_getlogin";

// 获取当前用户的登录名
pub fn utpam_modutil_check_user_in_passwd(utpamh: &mut Option<Box<UtpamHandle>>) -> Option<String> {
    let mut logname = None;

    // 尝试从数据中获取登录名
    let mut status = utpam_get_data(utpamh, Some(_PAMMODUTIL_GETLOGIN), &mut logname);
    if status == PAM_SUCCESS {
        if let Some(name) = logname {
            if let Some(s) = name.downcast_ref::<String>() {
                return Some(s.to_string());
            }
        }
    }

    // 获取当前用户的登录名
    let curr_user = match get_current_username() {
        Some(username) => username.to_string_lossy().into_owned(),
        None => return None,
    };

    // 将当前用户的登录名保存到数据中
    status = utpam_set_data(
        utpamh,
        Some(_PAMMODUTIL_GETLOGIN),
        Some(Rc::new(curr_user.clone())),
        Some(utpam_modutil_cleanup),
    );

    if status == PAM_SUCCESS {
        return None;
    }
    Some(curr_user)
}
