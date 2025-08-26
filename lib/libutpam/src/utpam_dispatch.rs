/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///模块调度
use crate::common::*;
use crate::utpam::*;
use crate::utpam_handlers::utpam_init_handlers;
pub fn utpam_dispatch(utpamh: &mut Option<Box<UtpamHandle>>, _flags: u32, _choice: i32) -> i32 {
    let retval = PAM_SYSTEM_ERR;

    let utpamh = match utpamh {
        Some(ref mut handle) => handle,
        None => {
            return PAM_SYSTEM_ERR;
        }
    };

    //具体的认证（待开发）

    //检查模块是否加载
    if utpam_init_handlers(utpamh) != PAM_SUCCESS {
        //失败，日志处理（待开发）
    }

    retval
}
