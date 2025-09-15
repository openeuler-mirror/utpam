/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::{
    PAM_INCOMPLETE, PAM_PRELIM_CHECK, PAM_SUCCESS, PAM_SYSTEM_ERR, PAM_UPDATE_AUTHTOK,
};
use crate::utpam::{
    UtpamBoolean, UtpamHandle, PAM_CALLED_FROM_MODULE, PAM_CHAUTHTOK, PAM_NOT_STACKED,
};
use crate::utpam_dispatch::utpam_dispatch;
use crate::utpam_misc::utpam_sanitize;
use crate::{IF_NO_UTPAMH, UTPAM_FROM_MODULE};

//管理密码或认证令牌的变更
pub fn utpam_chauthtok(utpamh: &mut Option<Box<UtpamHandle>>, flags: u32) -> i32 {
    ////检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_MODULE!(utpamh) {
        println!("called from module!?");
        return PAM_SYSTEM_ERR;
    }

    if flags & (PAM_PRELIM_CHECK | PAM_UPDATE_AUTHTOK) != 0 {
        println!("PAM_PRELIM_CHECK or PAM_UPDATE_AUTHTOK set by application");
        return PAM_SYSTEM_ERR;
    }

    if utpamh.former.choice == PAM_NOT_STACKED {
        utpam_sanitize(utpamh);
        utpamh.fail_delay.utpam_start_timer();
        utpamh.former.update = UtpamBoolean::UtpamFalse;
    }

    //模块调度
    let mut retval = utpam_dispatch(utpamh, flags | PAM_PRELIM_CHECK, PAM_CHAUTHTOK);

    if utpamh.former.update.to_bool() || retval != PAM_SUCCESS {
        println!("completed check ok: former={:?}", utpamh.former.update);
        utpamh.former.update = UtpamBoolean::UtpamTrue;
        retval = utpam_dispatch(utpamh, flags | PAM_UPDATE_AUTHTOK, PAM_CHAUTHTOK);
    }

    if retval != PAM_INCOMPLETE {
        utpam_sanitize(utpamh);
        utpamh.former.update = UtpamBoolean::UtpamFalse;
        println!("exiting {:?} - {:?}", retval, utpamh.former.choice);
    } else {
        println!("will resume when ready");
    }

    retval
}
