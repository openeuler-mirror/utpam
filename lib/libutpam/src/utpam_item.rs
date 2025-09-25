/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///设置和获取UtpamHandle结构体字段文件
use crate::common::*;
use crate::utpam::*;
use crate::utpam_delay::DelayFnPtr;
use crate::utpam_syslog::*;
use crate::{pam_syslog, utpam_overwrite_string, IF_NO_UTPAMH, UTPAM_FROM_MODULE};
use std::any::Any;
use std::rc::Rc;
use zeroize::Zeroize;

/// 更新UtpamHandle结构体字段
macro_rules! TRY_SET {
    ($old:expr, $item:expr, $field:ty) => {
        match $item {
            Some(item) => {
                if let Ok(data) = item.downcast::<$field>() {
                    if *data != $old {
                        $old = *data;
                    }
                } else {
                    return PAM_BAD_ITEM;
                }
            },
            None => {
                $old = <$field>::default(); // 将字段设置为默认值（对于字符串来说就是空字符串）
            }
        }
    };
}

// 设置UtpamHandle结构体中的特定项:
pub fn utpam_set_item(utpamh: &mut UtpamHandle, item_type: i8, item: Option<Box<dyn Any>>) -> u8 {
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
            TRY_SET!(utpamh.prompt, item, Option<String>);
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
                match item {
                    Some(item) => {
                        if let Ok(s) = item.downcast::<String>() {
                            if utpamh.authtok != *s {
                                utpam_overwrite_string!(utpamh.authtok);
                                utpamh.authtok = *s
                            }
                        }
                    }
                    None => {
                        utpamh.authtok = String::default();
                    }
                }
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_OLDAUTHTOK => {
            if UTPAM_FROM_MODULE!(utpamh) {
                match item {
                    Some(item) => {
                        if let Ok(s) = item.downcast::<String>() {
                            if utpamh.oldauthtok != *s {
                                utpam_overwrite_string!(utpamh.authtok);
                                utpamh.oldauthtok = *s
                            }
                        }
                    }
                    None => {
                        utpamh.oldauthtok = String::default();
                    }
                }
            } else {
                retval = PAM_BAD_ITEM;
            }
        }
        PAM_CONV => match item {
            Some(item) => {
                if let Ok(s) = item.downcast::<Rc<UtpamConv>>() {
                    utpamh.pam_conversation = Rc::clone(&s);
                    utpamh.former.fail_user = PAM_SUCCESS;
                }
            }
            None => {
                println!("pam_set_item: attempt to set conv() to NULL");
                retval = PAM_PERM_DENIED;
            }
        },
        PAM_FAIL_DELAY => {
            TRY_SET!(utpamh.fail_delay.delay_fn_ptr, item, Option<DelayFnPtr>);
        }
        PAM_XDISPLAY => {
            TRY_SET!(utpamh.xdisplay, item, String);
        }
        PAM_XAUTHDATA => {
            match item {
                Some(item) => {
                    if let Ok(s) = item.downcast::<UtpamXAuthData>() {
                        if utpamh.xauth != *s {
                            utpamh.xauth.clear(); // 清空xauth的旧数据
                            utpamh.xauth = *s
                        }
                    }
                }
                None => {
                    utpamh.xauth.clear();
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
pub fn utpam_get_item(utpamh: &UtpamHandle, item_type: i8, item: &mut Box<dyn Any>) -> u8 {
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

/// 获取用户名，并存储在utpamh 中。
pub fn utpam_get_user(
    utpamh: &mut Option<Box<UtpamHandle>>,
    user: &mut Option<String>,
    prompt: &mut Option<String>,
) -> u8 {
    let mut use_prompt = &mut String::new();

    //检查utpamh是否为空
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if !utpamh.user.is_empty() {
        *user = Some(utpamh.user.clone());
        return PAM_SUCCESS;
    }

    if utpamh.former.fail_user != PAM_SUCCESS {
        return utpamh.former.fail_user;
    }

    match prompt {
        Some(p) => {
            use_prompt = p;
        }
        None => {
            if let Some(ref mut prompt) = utpamh.prompt {
                *use_prompt = prompt.to_string();
            };
            *use_prompt = "login:".to_string();
        }
    }

    if utpamh.former.want_user.to_bool() {
        match utpamh.former.prompt {
            Some(ref mut former_prompt) => {
                if former_prompt != use_prompt {
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "utpam_get_user: resumed with different prompt",
                    );
                    return PAM_ABORT;
                }
                utpamh.former.want_user = UtpamBoolean::UtpamFalse;
                utpam_overwrite_string!(former_prompt);
            }
            None => {
                pam_syslog!(
                    &utpamh,
                    LOG_ERR,
                    "utpam_get_user: failed to resume with prompt",
                );
                return PAM_ABORT;
            }
        }
    }

    let mut msg = UtpamMessage {
        msg_style: 0,
        msg: String::default(),
    };
    msg.msg_style = PAM_PROMPT_ECHO_ON;
    msg.msg = use_prompt.to_string();
    let resp: &mut Option<Vec<UtpamResponse>> = &mut None;

    let conv = match utpamh.pam_conversation.conv {
        Some(ref conv) => conv,
        None => return PAM_CONV_ERR,
    };
    // 调用conv()函数，只获取1条消息
    let mut retval = conv(1, &[msg], resp, utpamh.pam_conversation.appdata_ptr.clone());

    match retval {
        PAM_SUCCESS | PAM_BUF_ERR | PAM_CONV_AGAIN | PAM_CONV_ERR => {}
        _ => retval = PAM_CONV_ERR,
    }
    match retval {
        PAM_CONV_AGAIN => {
            utpamh.former.want_user = UtpamBoolean::UtpamTrue;
            utpamh.former.prompt = Some(use_prompt.to_string());
        }
        PAM_SUCCESS => match resp {
            Some(res) => {
                if res[0].resp.is_empty() {
                    retval = utpam_set_item(utpamh, PAM_USER, Some(Box::new(res[0].resp.clone())));
                    *user = Some(utpamh.user.clone());
                }
            }
            None => {
                println!("no response provided");
                return PAM_CONV_ERR;
            }
        },
        _ => utpamh.former.fail_user = retval,
    }

    if resp.is_some() && retval != PAM_SUCCESS {
        pam_syslog!(
            &utpamh,
            LOG_WARNING,
            "unexpected response from failed conversation function",
        );
    };

    retval
}
