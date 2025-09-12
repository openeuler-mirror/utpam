/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///设置和获取UtpamHandle结构体字段文件
use crate::common::*;
use crate::utpam::*;
use crate::utpam_delay::DelayFnPtr;
use crate::{utpam_overwrite_string, UTPAM_FROM_MODULE};
use std::any::Any;
use std::rc::Rc;
use users::get_current_username;
use zeroize::Zeroize;

/// 更新UtpamHandle结构体字段
macro_rules! TRY_SET {
    ($old:expr, $item:expr, $field:ty) => {
        if let Ok(data) = $item.downcast::<$field>() {
            if *data != $old {
                $old = *data;
            }
        } else {
            return PAM_BAD_ITEM;
        }
    };
}

// 设置UtpamHandle结构体中的特定项:
pub fn utpam_set_item(utpamh: &mut UtpamHandle, item_type: i32, item: Box<dyn Any>) -> i32 {
    let mut retval = PAM_SUCCESS;

    match item_type {
        PAM_SERVICE => {
            utpamh.handlers.handlers_loaded = 0;
            TRY_SET!(utpamh.service_name, item, String);
            utpamh.service_name = utpamh.service_name.to_lowercase(); // 转换为小写
        }
        PAM_USER => {
            TRY_SET!(utpamh.user, item, String);
        }
        PAM_USER_PROMPT => {
            TRY_SET!(utpamh.prompt, item, String);
            utpamh.former.fail_user = PAM_SUCCESS;
        }
        PAM_TTY => {
            TRY_SET!(utpamh.tty, item, String);
        }
        PAM_RUSER => {
            TRY_SET!(utpamh.ruser, item, String);
        }
        PAM_RHOST => {
            TRY_SET!(utpamh.rhost, item, String);
        }
        PAM_AUTHTOK => {
            if UTPAM_FROM_MODULE!(utpamh) {
                if let Ok(s) = item.downcast::<String>() {
                    if utpamh.authtok != *s {
                        utpam_overwrite_string!(utpamh.authtok);
                        utpamh.authtok = *s
                    }
                }
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_OLDAUTHTOK => {
            if UTPAM_FROM_MODULE!(utpamh) {
                if let Ok(s) = item.downcast::<String>() {
                    if utpamh.oldauthtok != *s {
                        utpam_overwrite_string!(utpamh.authtok);
                        utpamh.oldauthtok = *s
                    }
                }
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_CONV => {
            if let Ok(s) = item.downcast::<Rc<UtpamConv>>() {
                utpamh.pam_conversation = Rc::clone(&s);
                utpamh.former.fail_user = PAM_SUCCESS;
            } else {
                retval = -1;
            }
        }
        PAM_FAIL_DELAY => {
            TRY_SET!(utpamh.fail_delay.delay_fn_ptr, item, Option<DelayFnPtr>);
        }
        PAM_XDISPLAY => {
            TRY_SET!(utpamh.xdisplay, item, String);
        }
        PAM_XAUTHDATA => {
            if let Ok(s) = item.downcast::<UtpamXAuthData>() {
                if utpamh.xauth != *s {
                    utpamh.xauth.clear(); // 清空xauth的旧数据
                    utpamh.xauth = *s
                }
            }
        }
        PAM_AUTHTOK_TYPE => {
            TRY_SET!(utpamh.authtok_type, item, String);
        }
        _ => retval = PAM_BAD_ITEM,
    }

    retval
}

/* 从UtpamHandle结构体中获取特定项并存储在item中:
 *   1. 使用Box<dyn Any>存储不同类型的值；
 *   2. 使用 .clone() 方法克隆字符串和其他类型的数据，确保不会影响原始数据；
 *   3. 对于无法克隆的数据，使用Rc<T>来包装数据，并使用 .clone() 方法获取一个引用，而不是复制数据。
 */
pub fn utpam_get_item(utpamh: &UtpamHandle, item_type: i32, item: &mut Box<dyn Any>) -> i32 {
    let mut retval = PAM_SUCCESS;

    // 根据item_type选择要返回的数据
    match item_type {
        PAM_SERVICE => {
            *item = Box::new(utpamh.service_name.clone());
        }
        PAM_USER => {
            *item = Box::new(utpamh.user.clone());
        }
        PAM_USER_PROMPT => {
            *item = Box::new(utpamh.prompt.clone());
        }
        PAM_TTY => {
            *item = Box::new(utpamh.tty.clone());
        }
        PAM_RUSER => {
            *item = Box::new(utpamh.ruser.clone());
        }
        PAM_RHOST => {
            *item = Box::new(utpamh.rhost.clone());
        }
        PAM_AUTHTOK => {
            if UTPAM_FROM_MODULE!(utpamh) {
                *item = Box::new(utpamh.authtok.clone());
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_OLDAUTHTOK => {
            if UTPAM_FROM_MODULE!(utpamh) {
                *item = Box::new(utpamh.oldauthtok.clone());
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_CONV => {
            *item = Box::new(Rc::clone(&utpamh.pam_conversation));
        }
        PAM_FAIL_DELAY => {
            *item = Box::new(utpamh.fail_delay.delay_fn_ptr);
        }
        PAM_XDISPLAY => {
            *item = Box::new(utpamh.xdisplay.clone());
        }
        PAM_XAUTHDATA => {
            *item = Box::new(utpamh.xauth.clone());
        }
        PAM_AUTHTOK_TYPE => {
            *item = Box::new(utpamh.authtok_type.clone());
        }
        _ => retval = PAM_BAD_ITEM,
    }

    retval
}

/// 获取当前用户名
pub fn get_username() -> String {
    match get_current_username() {
        Some(username) => username.to_string_lossy().into_owned(),
        None => {
            "login:".to_string() // 如果获取用户名失败，返回默认值
        }
    }
}
