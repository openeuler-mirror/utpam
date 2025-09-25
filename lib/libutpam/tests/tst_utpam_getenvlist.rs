/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::rc::Rc;
use utpam::common::{UtpamConv, PAM_SUCCESS};
use utpam::utpam::UtpamHandle;
use utpam::utpam_env::utpam_getenvlist;
use utpam::utpam_env::utpam_putenv;
use utpam::utpam_start::utpam_start;

const ENVVALS: [&str; 3] = ["VAL1=1", "VAL2=2", "VAL3=3"];

fn tst_utpam_getenvlist() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = Rc::new(UtpamConv {
        conv: None,
        appdata_ptr: None,
    });
    let mut utpamh: Option<Box<UtpamHandle>> = None;
    let mut ptr: Option<Vec<String>>;
    let mut retval: u8;

    /* 1: Call with NULL as pam handle */
    ptr = utpam_getenvlist(&mut None);
    if !ptr.is_none() {
        println!("utpam_getenvlist (NULL) does not return NULL");
        return 1;
    }

    /* setup pam handle */
    retval = utpam_start(
        service.clone(),
        Some(user.clone()),
        Some(conv.clone()),
        &mut utpamh,
    );
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &pamh) returned {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    }

    /* 2: Call with pam handle, but no environment set */
    ptr = utpam_getenvlist(&mut utpamh);
    if ptr.is_none() {
        println!("utpam_getenvlist (pamh) does not return pointer to NULL");
    }

    /* set environment variable */
    for i in 0..3 {
        retval = utpam_putenv(&mut utpamh, ENVVALS[i]);
        if retval != PAM_SUCCESS {
            println!("utpam_putenv (pamh, {}) returned {}", ENVVALS[i], retval);
            return 1;
        }
    }

    /* 3: Call with pam handle and environment set */
    ptr = utpam_getenvlist(&mut utpamh);
    match ptr {
        Some(ptr) => {
            for (i, item) in ptr.iter().enumerate() {
                if item != &ENVVALS[i] {
                    println!(
                        "utpam_getenvlist returns wrong value: expected: {},got: {}",
                        ENVVALS[i], item
                    );
                    return 1;
                }
            }
        }
        None => {
            println!("utpam_getenvlist (pamh) returned NULL");
            return 1;
        }
    }

    0
}

#[test]
fn test_utpam_get_user() {
    assert_eq!(tst_utpam_getenvlist(), 0);
}
