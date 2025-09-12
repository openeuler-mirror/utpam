/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_assignments)]
#![allow(clippy::borrow_interior_mutable_const)]

use crate::utpam::UtpamHandle;
use colored::*;
use tklog::{LOG, MODE};

//const PAM_NOT_STACKED: i8 = 0;
const PAM_AUTHENTICATE: i8 = 1;
const PAM_SETCRED: i8 = 2;
const PAM_ACCOUNT: i8 = 3;
const PAM_OPEN_SESSION: i8 = 4;
const PAM_CLOSE_SESSION: i8 = 5;
const PAM_CHAUTHTOK: i8 = 6;

//日志存放在当前目录下，pam.log
//之后可以更改该参数，比如/var/log/secure
const PAM_LOG_FILE: &str = "pam.log";

pub fn log_init() {
    let levelstr = "{level}".green(); //日志级别标识设置为绿色
    let timestr = "{time}".yellow(); // 时间属性标识设置为黄色
    let filestr = "{file}".red(); //文件属性标识设置为红色
    let messagestr = ":{message}".blue(); // 信息属性标识修改为蓝色
    let s = format!("{} {} {} {}\n", levelstr, timestr, filestr, messagestr);
    //设置日志格式
    LOG.set_formatter(s.as_str());
    // 日志写入到文件
    LOG.set_cutmode_by_time(PAM_LOG_FILE, MODE::MONTH, 0, false);
}

fn _pam_choice2str(choice: i8) -> &'static str {
    match choice {
        PAM_AUTHENTICATE => "auth",
        PAM_SETCRED => "setcred",
        PAM_ACCOUNT => "account",
        PAM_OPEN_SESSION => "session",
        PAM_CLOSE_SESSION => "session",
        PAM_CHAUTHTOK => "chauthtok",
        _ => "",
    }
}

#[macro_export]
macro_rules! pam_syslog {
    ($utpamh:expr, $priority:expr, $fmt:expr, $($args:tt),*) => {{

        let mut msgbuf2:String = String::new();
        msgbuf2.push_str($fmt);
        $(msgbuf2.push_str($args);)*

        match $priority {
            LOG_EMERG | LOG_ALERT |LOG_CRIT => {
                fatal!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
            LOG_ERR => {
                error!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
            LOG_WARNING => {
                warn!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
            LOG_NOTICE | LOG_INFO => {
                info!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
            LOG_DEBUG => {
                debug!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
            _ => {
                debug!(utpam_patching_msgbuf1($utpamh), msgbuf2);
            }
        }
    }};
}

//字符串拼接函数,从utpamh中获取数据，返回拼接后的字符串。
// eg: pam_unix(sshd:session):
// eg: pam_unix(sudo:session):
pub fn utpam_patching_msgbuf1(utpamh: &UtpamHandle) -> String {
    let mut msgbuf1: String = String::new();
    let temp_string: String = if !utpamh.service_name.is_empty() {
        utpamh.service_name.to_string()
    } else {
        "<unknown>".to_string()
    };

    msgbuf1 = format!(
        "{}({}:{}):",
        &utpamh.mod_name,
        temp_string,
        _pam_choice2str(utpamh.choice as i8)
    );
    msgbuf1
}
