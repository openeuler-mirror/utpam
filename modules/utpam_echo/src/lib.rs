/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut)]
#![allow(unused_variables)]

use std::{any::Any, fs::OpenOptions, os::unix::prelude::MetadataExt};
use utpam::common::*;
use utpam::utpam::*;
use utpam::utpam_item::utpam_get_item;
use utpam::utpam_modutil_ioloop::utpam_modutil_read;

//替换消息中的占位符，并将最终的消息输出到日志中
fn replace_and_print(utpamh: &UtpamHandle, mesg: &str) -> u8 {
    let mut output = Vec::with_capacity(mesg.len() + PAM_MAX_MSG_SIZE); // 为输出字符串预留足够空间
    let mut p = mesg.chars().peekable();
    let mut item = 0;
    let mut s: Box<dyn Any> = Box::new(());
    let mut str = None;

    while let Some(c) = p.next() {
        if c != '%' || p.peek().is_none() {
            output.push(c.to_string());
            continue;
        }
        if let Some(next) = p.next() {
            match next {
                'H' => {
                    item = PAM_RHOST;
                }
                'h' => {
                    item = -2;
                }
                's' => {
                    item = PAM_SERVICE;
                }
                't' => {
                    item = PAM_TTY;
                }
                'U' => {
                    item = PAM_RUSER;
                }
                'u' => {
                    item = PAM_USER;
                }
                _ => {
                    output.push(c.to_string());
                    continue;
                }
            }
        }
        match item {
            -2 => {
                if let Ok(hostname) = hostname::get() {
                    if let Ok(host_str) = hostname.into_string() {
                        str = Some(host_str);
                    }
                }
            }
            _ => {
                if utpam_get_item(utpamh, 1, &mut s) == 0 {
                    if let Some(s) = s.downcast_ref::<String>() {
                        str = Some(s.clone());
                    }
                }
            }
        }
        if str.is_none() {
            str = Some("(null)".to_string());
        }
        if let Some(ref s) = str {
            for c in s.chars() {
                output.push(c.to_string());
            }
        }
        str = None;
    }

    //pam_info (pamh, "%s", output); 将消息输出到日志，待开发
    println!("output: {:?}", output.join(""));

    PAM_SUCCESS
}

//在 PAM 认证过程的不同阶段输出指定的消息
fn utpam_echo(
    mut utpamh: &mut Option<Box<UtpamHandle>>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    let mut file: Option<String> = None;
    let mut retval;

    let utpamh = match utpamh {
        Some(ref pamh) => pamh,
        None => return PAM_SYSTEM_ERR,
    };

    if flags & PAM_SILENT != 0 {
        return PAM_IGNORE;
    }

    //遍历传递的参数，查找 file= 选项
    if let Some(ref args) = argv {
        for arg in args.iter() {
            if let Some(result) = arg.strip_prefix("file=") {
                if !result.trim().is_empty() {
                    file = Some(result.to_string());
                }
            }
        }
    }

    match file {
        //如果传递了 file= 选项，则读取指定的文件内容，并将其作为消息输出
        Some(f) => {
            let mut file = OpenOptions::new().read(true).open(f);
            match file {
                Ok(ref mut fd) => {
                    let size = fd.metadata().unwrap().size();
                    if size == 0 {
                        return PAM_IGNORE;
                    }
                    if size > isize::MAX as u64 {
                        return PAM_BUF_ERR;
                    }

                    let mut buffer = vec![0; size as usize];

                    match utpam_modutil_read(fd, &mut buffer) {
                        Ok(bytes_read) => {
                            let content = String::from_utf8_lossy(&buffer[..bytes_read]);
                            println!("{}", content);
                            retval = replace_and_print(utpamh, &content);
                        }
                        Err(e) => {
                            println!("read error: {}", e);
                            return PAM_IGNORE;
                        }
                    }
                }
                Err(e) => {
                    println!("open error: {}", e);
                    retval = PAM_IGNORE;
                }
            }
        }

        //如果没有传递 file= 选项，或者传递的值为空，则使用默认的 echo 输出
        None => {
            let mut msg = String::with_capacity(PAM_MAX_MSG_SIZE);
            for (i, arg) in argv.iter().flatten().enumerate() {
                if i > 0 && msg.len() + 1 < PAM_MAX_MSG_SIZE {
                    msg.push(' '); // 添加空格分隔参数
                }
                for c in arg.chars() {
                    if msg.len() < PAM_MAX_MSG_SIZE {
                        msg.push(c); // 将参数内容复制到 msg 中
                    }
                }
            }
            retval = replace_and_print(utpamh, &msg);
        }
    }

    retval
}

//用户认证
#[no_mangle]
pub fn utpam_sm_authenticate(
    mut utpamh: &mut Option<Box<UtpamHandle>>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    utpam_echo(utpamh, flags, argc, argv)
}

//设置凭证
#[no_mangle]
pub fn utpam_sm_setcred(
    mut _utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_IGNORE
}

//账户管理
pub fn utpam_sm_acct_mgmt(
    mut utpamh: &mut Option<Box<UtpamHandle>>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    utpam_echo(utpamh, flags, argc, argv)
}

//打开会话
pub fn utpam_sm_open_session(
    mut utpamh: &mut Option<Box<UtpamHandle>>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    utpam_echo(utpamh, flags, argc, argv)
}

//关闭会话
pub fn utpam_sm_close_session(
    mut _utpamh: &mut Option<Box<UtpamHandle>>,
    _flags: i32,
    _argc: Option<i32>,
    _argv: Option<Vec<String>>,
) -> u8 {
    PAM_IGNORE
}

//密码管理
pub fn utpam_sm_chauthtok(
    mut utpamh: &mut Option<Box<UtpamHandle>>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8 {
    if flags & PAM_PRELIM_CHECK != 0 {
        utpam_echo(utpamh, flags, argc, argv)
    } else {
        PAM_IGNORE
    }
}
