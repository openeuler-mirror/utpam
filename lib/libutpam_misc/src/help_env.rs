/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::ffi::CString;
use utpam::common::{PAM_BUF_ERR, PAM_PERM_DENIED, PAM_SUCCESS};
use utpam::utpam::UtpamHandle;
use utpam::utpam_env::{utpam_getenv, utpam_putenv};

#[cfg(feature = "debug")]
use utpam::common::{utpam_output_debug, utpam_output_debug_info};
#[cfg(feature = "debug")]
use utpam::utpam_strerror::utpam_strerror;
use utpam::D;

#[no_mangle]
pub fn utpam_misc_paste_env(utpamh: &mut Option<Box<UtpamHandle>>, user_env: &[&str]) -> u8 {
    for env_var in user_env.iter().filter(|s| !s.is_empty()) {
        D!("uploading: {}", env_var);
        let retval: u8 = utpam_putenv(utpamh, env_var);
        if retval != PAM_SUCCESS {
            D!("error setting {:?}: {:?}", env_var, utpam_strerror(retval));
            return retval;
        }
    }
    D!("done.");
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
    utpamh: &mut Option<Box<UtpamHandle>>,
    name: &str,
    value: &str,
    readonly: i64,
) -> u8 {
    if readonly != 0 {
        let etmp: Option<String> = utpam_getenv(utpamh, name);
        if etmp.is_some() {
            D!("failed to set readonly variable: {}", name);
            return PAM_PERM_DENIED;
        }
    }

    let tmp = format!("{}={}", name, value);
    let retval: u8 = if !tmp.is_empty() {
        D!("pam_putt()ing: {}", tmp);
        utpam_putenv(utpamh, &tmp)
    } else {
        D!("malloc failure");
        PAM_BUF_ERR
    };
    retval
}
