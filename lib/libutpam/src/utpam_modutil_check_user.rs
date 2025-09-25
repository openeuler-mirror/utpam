/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::*;
use crate::pam_syslog;
use crate::utpam::*;
use crate::utpam_syslog::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

// 检查用户名是否在指定的文件中存在
pub fn utpam_modutil_check_user_in_passwd(
    utpamh: &mut Box<UtpamHandle>,
    user_name: Option<&str>,
    file_name: Option<&str>,
) -> u8 {
    let user_name = match user_name {
        Some(f) => f,
        None => {
            pam_syslog!(utpamh, LOG_NOTICE, "user name is not valid",);
            return PAM_SERVICE_ERR;
        }
    };

    if user_name.contains(':') {
        return PAM_PERM_DENIED;
    }

    // 如果file_name为空，则使用默认密码文件
    let file_name = file_name.unwrap_or("/etc/passwd");

    // 打开文件
    let file = match File::open(file_name) {
        Ok(file) => file,
        Err(err) => {
            pam_syslog!(utpamh, LOG_ERR, "error opening {}: {}", file_name, err);
            return PAM_SERVICE_ERR;
        }
    };

    let mut rc = PAM_PERM_DENIED;
    let reader = BufReader::new(file);
    // 逐行读取文件
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                pam_syslog!(utpamh, LOG_ERR, "error reading : {}", err);
                return PAM_SERVICE_ERR;
            }
        };

        // 检查当前行是否以 user_name 开头，并且 user_name 后面跟着一个冒号
        if line.starts_with(user_name) && line[user_name.len()..].starts_with(':') {
            rc = PAM_SUCCESS;
        }
    }

    rc
}
