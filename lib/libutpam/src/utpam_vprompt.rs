/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_assignments)]
#![allow(dead_code)]

use crate::utpam::UtpamHandle;
use crate::utpam_item::utpam_get_item;
use crate::utpam_overwrite_string;
use crate::utpam_syslog::*;
use crate::{common::*, pam_syslog};
use std::any::Any;
use std::rc::Rc;
use zeroize::Zeroize;

#[macro_export]
macro_rules! pam_error {
    ($utpamh:expr, $fmt:expr, $($args:tt),*) => {{
        let msgbuf = format!($fmt, $($args),*);
        pam_vprompt($utpamh, PAM_ERROR_MSG, Vec::new(), msgbuf);
    }
}
}

#[macro_export]
macro_rules! pam_info {
    ($utpamh:expr, $fmt:expr, $($args:tt),*) => {{
        let msgbuf = format!($fmt, $($args),*);
        pam_vprompt($utpamh, PAM_TEXT_INFO, Vec::new(), msgbuf);
    }
}
}

#[macro_export]
macro_rules! pam_prompt {
    ($utpamh:expr, $style:expr, $response:expr,$fmt:expr, $($args:tt),*) => {{
        let msgbuf = format!($fmt, $($args),*);
        pam_vprompt($utpamh, $style, $response, msgbuf)
    }
}
}

pub fn pam_vprompt(
    utpamh: &UtpamHandle,
    style: u8,
    mut response: Vec<String>,
    msgbuf: String,
) -> u8 {
    let mut msg = UtpamMessage {
        msg_style: style,
        msg: String::default(),
    };
    let mut pam_resp: Option<Vec<UtpamResponse>> = None;
    let mut convp: Box<dyn Any> = Box::new(());
    let mut retval: u8 = 0;

    if response.is_empty() {
        response = Vec::default();
    }

    retval = utpam_get_item(utpamh, PAM_CONV, &mut convp);
    if retval != PAM_SUCCESS {
        return retval;
    }
    let conv: &UtpamConv = match convp.downcast_ref::<Rc<UtpamConv>>() {
        Some(item) => item,
        None => {
            return PAM_SYSTEM_ERR;
        }
    };
    let new_conv = match conv.conv {
        Some(ref conv) => conv,
        None => {
            pam_syslog!(utpamh, LOG_ERR, "no conversation function",);
            return PAM_SYSTEM_ERR;
        }
    };

    if msgbuf.is_empty() {
        pam_syslog!(utpamh, LOG_ERR, "empty message",);
        return PAM_CONV_ERR;
    }
    msg.msg = msgbuf.clone();
    // 调用conv()函数，只获取1条消息
    retval = new_conv(1, &[msg], &mut pam_resp, conv.appdata_ptr.clone());
    if retval != PAM_SUCCESS && pam_resp.is_some() {
        pam_syslog!(
            utpamh,
            LOG_WARNING,
            "unexpected response from failed conversation function",
        );
    }
    if !response.is_empty() {
        match pam_resp {
            Some(ref mut resp) => {
                response = resp[0].resp.clone();

                if !resp[0].resp.is_empty() {
                    utpam_overwrite_string!(resp[0].resp);
                }
            }
            None => response.push("".to_string()),
        }
    }
    utpam_overwrite_string!(msgbuf.clone());

    if retval != PAM_SUCCESS {
        pam_syslog!(utpamh, LOG_ERR, "conversation failed",);
    }

    retval
}
