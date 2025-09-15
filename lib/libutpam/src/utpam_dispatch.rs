/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(
    unused_variables,
    dead_code,
    unused_assignments,
    clippy::collapsible_if
)]
///模块调度
use crate::common::*;
use crate::utpam::*;
use crate::utpam_handlers::utpam_init_handlers;
use crate::utpam_item::utpam_get_item;

use crate::{UTPAM_FROM_MODULE, UTPAM_TO_APP, UTPAM_TO_MODULE};

use std::any::Any;

const PAM_UNDEF: i32 = 0;
const PAM_POSITIVE: i32 = 1;
const PAM_NEGATIVE: i32 = -1;

const PAM_PLEASE_FREEZE: i32 = 0;
const PAM_MAY_BE_FROZEN: i32 = 1;
const PAM_MUST_BE_FROZEN: i32 = 2;

const PAM_MUST_FAIL_CODE: i32 = PAM_PERM_DENIED;

/// 遍历和调度配置的模块堆栈
fn utpam_dispatch_aux(
    utpamh: &mut Box<UtpamHandle>,
    flags: u32,
    handlers: &mut Option<Box<Handler>>,
    resumed: UtpamBoolean,
    use_cached_chain: i32,
) -> i32 {
    let depth = 0;
    let mut impression = PAM_UNDEF;
    let mut status = PAM_MUST_FAIL_CODE;
    let mut skip_depth = 0;
    let mut prev_level = 0;
    let stack_level = 0;

    let mut substates = vec![UtpamSubstackState {
        impression: PAM_UNDEF,
        status: PAM_MUST_FAIL_CODE,
    }];

    if handlers.is_none() {
        let mut service: Box<dyn Any> = Box::new(());

        // 获取服务名
        utpam_get_item(utpamh, PAM_SERVICE, &mut service);

        // 因为service_name是String, 将 Box<dyn Any> 转换为 String 类型的引用
        if let Some(name) = service.downcast_ref::<String>() {
            println!("no modules loaded for '{}' service", name);
        } else {
            println!("no modules loaded for <unknown> service");
        }
        return PAM_MUST_FAIL_CODE;
    }
    println!("resumed: {:?}", resumed);
    if resumed.to_bool() {
        skip_depth = utpamh.former.depth;
        status = utpamh.former.status;
        impression = utpamh.former.impression;
        substates = utpamh.former.substates.clone();

        //清空 pamh->former 中的状态，为下次可能的恢复调用做准备
        utpamh.former.impression = PAM_UNDEF;
        utpamh.former.status = PAM_MUST_FAIL_CODE;
        utpamh.former.depth = 0;
        utpamh.former.substates = vec![];
    };

    prev_level = 0;

    //待开发

    if status == PAM_SUCCESS && impression != PAM_POSITIVE {
        status = PAM_MUST_FAIL_CODE;
    }

    status
}

/// 重置所有模块的grantor标记，确保模块堆栈的每一次执行都是干净的
fn utpam_clear_grantors(handler: &mut Option<Box<Handler>>) {
    for handler in handler.iter_mut() {
        handler.grantor = 0;
    }
}

/// 将模块调度请求转换为指向将实际运行的模块堆栈的指针
pub fn utpam_dispatch(utpamh: &mut Box<UtpamHandle>, flags: u32, choice: i32) -> i32 {
    let mut retval = PAM_SYSTEM_ERR;
    let mut use_cached_chain;
    let mut h;
    let resumed;

    if UTPAM_FROM_MODULE!(utpamh) {
        return retval;
    }

    //检查模块是否加载
    if utpam_init_handlers(utpamh) != PAM_SUCCESS {
        println!("unable to dispatch function");
        return retval;
    }

    // 控制模块栈的执行方式
    use_cached_chain = PAM_PLEASE_FREEZE;

    //选择认证处理器
    match choice {
        PAM_AUTHENTICATE => {
            h = &mut utpamh.handlers.conf.authenticate;
        }
        PAM_SETCRED => {
            h = &mut utpamh.handlers.conf.setcred;
            use_cached_chain = PAM_MAY_BE_FROZEN;
        }
        PAM_ACCOUNT => {
            h = &mut utpamh.handlers.conf.acct_mgmt;
        }
        PAM_OPEN_SESSION => {
            h = &mut utpamh.handlers.conf.open_session;
        }
        PAM_CLOSE_SESSION => {
            h = &mut utpamh.handlers.conf.close_session;
            use_cached_chain = PAM_MAY_BE_FROZEN;
        }
        PAM_CHAUTHTOK => {
            h = &mut utpamh.handlers.conf.chauthtok;
        }
        _ => {
            return PAM_ABORT;
        }
    };

    // 如果conf中没有找到对应的处理器，则使用other中的默认处理器
    if h.is_none() {
        match choice {
            PAM_AUTHENTICATE => {
                h = &mut utpamh.handlers.other.authenticate;
            }
            PAM_SETCRED => {
                h = &mut utpamh.handlers.other.setcred;
            }
            PAM_ACCOUNT => {
                h = &mut utpamh.handlers.other.acct_mgmt;
            }
            PAM_OPEN_SESSION => {
                h = &mut utpamh.handlers.other.open_session;
            }
            PAM_CLOSE_SESSION => {
                h = &mut utpamh.handlers.other.close_session;
            }
            PAM_CHAUTHTOK => {
                h = &mut utpamh.handlers.other.chauthtok;
            }
            _ => {}
        };
    }

    /* 处理PAM_NOT_STACKED状态：
     *	如果上次调用返回了"不完整状态"，则需要恢复状态
     *   如果上次调用与本次调用的选择不同，则视为错误
     */
    if utpamh.former.choice != PAM_NOT_STACKED {
        if utpamh.former.choice != choice {
            return PAM_ABORT;
        }
        resumed = UtpamBoolean::UtpamTrue;
    } else {
        resumed = UtpamBoolean::UtpamFalse;

        //清除授予者（grantors）的信息
        utpam_clear_grantors(h);
    }

    // 将上下文转换为模块上下文,为了调用模块函数做准备
    UTPAM_TO_MODULE!(utpamh);

    // 处理模块函数列表
    utpamh.former.choice = choice;
    let mut h_local = h.clone();
    retval = utpam_dispatch_aux(utpamh, flags, &mut h_local, resumed, use_cached_chain);

    // 模块函数调用完成后返回到应用程序
    UTPAM_TO_APP!(utpamh);

    // 等待应用程序处理完后重新开始
    if retval == PAM_INCOMPLETE {
        // 记录需要在下次调用时恢复的状态。
        println!("module [%s] returned PAM_INCOMPLETE");
        utpamh.former.choice = choice;
    } else {
        //清除上次调用的状态
        utpamh.former.choice = PAM_NOT_STACKED;
    }

    retval
}
