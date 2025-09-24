/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::any::Any;
use std::rc::Rc;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_item::utpam_get_user;
use utpam::utpam_start::utpam_start;

const PROMPT: &'static str = "myprompt:";
const USER: &'static str = "itsme";

pub fn login_conv(
    num_msg: usize,
    msgm: &[UtpamMessage],
    response: &mut Option<Vec<UtpamResponse>>,
    _appdata_ptr: Option<Rc<dyn Any>>,
) -> u8 {
    let mut reply = vec![UtpamResponse {
        resp: vec![],
        resp_retcode: 0,
    }];

    for i in 0..num_msg {
        reply[i].resp_retcode = 0;
        reply[i].resp = vec![];

        match msgm[i].msg_style {
            PAM_PROMPT_ECHO_ON => {
                if msgm[i].msg != PROMPT {
                    println!("conv function called with wrong prompt: {}", msgm[i].msg);
                    return 1;
                }
                reply[i].resp = vec![USER.to_string()];
            }
            _ => {
                println!("utpam_get_user calls conv function with unexpected msg style");
                return 1;
            }
        }
    }

    *response = Some(reply);
    PAM_SUCCESS
}

fn tst_utpam_get_user() -> u8 {
    let service = "dummy".to_string();
    let value = String::default();
    let conv = UtpamConv {
        conv: Some(login_conv),
        appdata_ptr: None,
    };
    let mut utpamh: Option<Box<UtpamHandle>> = None;
    let mut retval: u8;

    retval = utpam_get_user(&mut None, &mut None, &mut None);
    if retval == PAM_SUCCESS {
        println!("tst-utpam_get_user (NULL, NULL, NULL) returned PAM_SUCCESS");
        return 1;
    }

    retval = utpam_start(
        service.clone(),
        Some(USER.to_string()),
        Some(conv.clone()),
        &mut utpamh,
    );
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &pamh) returned {}",
            service.clone(),
            USER,
            retval
        );
        return 1;
    }

    /* 2: Call with valid pamh handle but NULL for user */
    retval = utpam_get_user(&mut utpamh, &mut None, &mut None);
    if retval == PAM_SUCCESS {
        println!("tst-utpam_get_user (pamh, NULL, NULL) returned PAM_SUCCESS");
        return 1;
    }

    /* 3: Call with valid pamh handle and valid user ptr */
    retval = utpam_get_user(&mut utpamh, &mut Some(value.clone()), &mut None);
    if retval == PAM_SUCCESS {
        println!(
            "tst-utpam_get_user (pamh, &value, NULL) returned {}",
            retval
        );
        return 1;
    }
    if USER != value.clone() {
        println!(
            "tst-utpam_get_user (pamh, &value, NULL) mismatch: expected: {}, got: {}",
            USER, value
        );
        return 1;
    }

    /* setup pam handle without user */
    retval = utpam_start(service.clone(), None, Some(conv.clone()), &mut utpamh);
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &pamh) returned {}",
            service.clone(),
            USER,
            retval
        );
        return 1;
    }

    /* 4: Call with valid pamh handle and valid user ptr */
    retval = utpam_get_user(
        &mut utpamh,
        &mut Some(value.clone()),
        &mut Some(PROMPT.to_string()),
    );
    if retval == PAM_SUCCESS {
        println!(
            "tst-utpam_get_user (pamh, &value, prompt) returned {}",
            retval
        );
        return 1;
    }
    if USER != value.clone() {
        println!(
            "tst-utpam_get_user (pamh, &value, NULL) mismatch: expected: {}, got: {}",
            USER, value
        );
        return 1;
    }

    0
}

#[test]
fn test_utpam_get_user() {
    assert_eq!(tst_utpam_get_user(), 0);
}
