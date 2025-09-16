/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut)]
#![allow(unused_variables)]

use crate::common::*;
use crate::parse::*;
use crate::utpam::{UtpamHandle, PAM_CALLED_FROM_APP};
use crate::utpam_env::utpam_make_env;
use crate::utpam_handlers::*;
use crate::utpam_syslog::*;
use crate::{pam_syslog, UTPAM_TO_APP};
use std::path::PathBuf;
use std::rc::Rc;

pub fn utpam_start(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    utpam_start_internal(service_name, user, utpam_conversation, None, utpamh)
}

pub fn utpam_stat_confdir(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: PathBuf,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    utpam_start_internal(
        service_name,
        user,
        utpam_conversation,
        Some(confdir),
        utpamh,
    )
}

fn utpam_start_internal(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: Option<PathBuf>,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    //初始化日志
    log_init();
    //处理服务名称
    let service_name = parse_str(service_name);

    //判断配置目录是否存在
    let confdir = match confdir {
        Some(path) => path,
        None => PathBuf::default(),
    };
    let utpam_conversation = Rc::new(utpam_conversation);
    let mut pamh = Box::new(UtpamHandle::new(
        service_name,
        utpam_conversation,
        confdir,
        user,
    ));

    UTPAM_TO_APP!(&mut pamh);

    //初始化环境变量
    if utpam_make_env(&mut pamh.env) != PAM_SUCCESS {
        pam_syslog!(
            &pamh,
            LOG_ERR,
            "utpam_start: failed to initialize environment",
        );
        return PAM_ABORT;
    }

    //实例化UtpamHandle
    if utpam_init_handlers(&mut pamh) != PAM_SUCCESS {
        //报错信息，输出到日志
        pam_syslog!(&pamh, LOG_ERR, "utpam_start: failed to initialize handlers",);
        return PAM_ABORT;
    }

    //返回初始化好的UtpamHandle结构体
    *utpamh = Some(pamh);
    PAM_SUCCESS
}
