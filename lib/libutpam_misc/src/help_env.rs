/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_imports,
    clippy::len_zero,
    clippy::unnecessary_cast
)]

use std::ffi::CString;
use utpam::common::PAM_BUF_ERR;
use utpam::common::PAM_PERM_DENIED;
use utpam::common::PAM_SUCCESS;
use utpam::utpam::UtpamHandle;
use utpam::utpam_strerror::pam_strerror;

//将用户环境变量复制到utpamh，判断用户环境变量指针及指针内容是否为空，如不为空，使用
//pam_putenv函数添加到pamh中；
//spi；
//遗留部分为debug，fn pam_putenv,fn pam_strerror,
#[no_mangle]
pub fn utpam_misc_paste_env(utpamh: &mut Box<UtpamHandle>, user_env: &[&str]) -> u8 {
    for env_var in user_env.iter().filter(|s| !s.is_empty()) {
        let mut retval: u8 = 0;

        println!("uploading: {}", env_var); //debug
                                            //retval = pam_putenv(pamh,env_var);
        if retval != PAM_SUCCESS {
            println!(
                "error setting {:?}: {:?}",
                env_var,
                pam_strerror(utpamh, retval)
            ); //debug
            return retval;
        }
    }
    println!("done."); //debug
    PAM_SUCCESS
}

/*
 * 原函数功能：传入二维指针，指针指向数组，目的为清除内容且释放内存；
 * spi;
 * 重构函数忽略了传入的参数内容，结合生命周期等rust特性，程序结束会释放内存等，
 * 所以此处不需要进行释放相关操作，直接返回空；
*/
#[no_mangle]
pub fn utpam_misc_drop_env() -> Vec<CString> {
    Vec::new()
}

/*
 *在pam环境中设置环境变量，如果readonly只读，且utpamh中已有name这个环境变量，
 *则表示name这个变量由于只读，无法重置。
 *如readonly非只读，则组包“name=value”,之后添加进utpamh,释放资源。
 *spi;
 * */
#[no_mangle]
pub fn utpam_misc_setenv(
    utpamh: &mut Box<UtpamHandle>,
    name: &'static str,
    value: &'static str,
    readonly: i64,
) -> u8 {
    let mut tmp: String = Default::default();
    let mut retval: u8 = 0;

    if readonly != 0 {
        let etmp: Option<&'static str> = None;
        //etmp = pam_getenv(utpamh,name);
        if let Some(s) = etmp {
            println!("failed to set readonly variable: {}", name); //debug
            return PAM_PERM_DENIED;
        }
    }

    let result = format!("{}={}", name, value);
    if result.len() >= 1 {
        println!("pam_putt()ing: {}", tmp); //debug
                                            //retval = pam_putenv(utpamh,result);
                                            //关于pam_drop,result离开作用域会自动释放；
    } else {
        println!("malloc failure"); //debug
        retval = PAM_BUF_ERR;
    }
    retval
}
