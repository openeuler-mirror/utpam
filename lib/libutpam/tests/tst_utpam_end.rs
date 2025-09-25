/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::rc::Rc;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_end::utpam_end;
use utpam::utpam_start::utpam_start;
fn tst_utpam_end() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = Rc::new(UtpamConv {
        conv: None,
        appdata_ptr: None,
    });
    let mut utpamh: Option<Box<UtpamHandle>> = None;

    /* 1: check with valid arguments */
    let mut retval = utpam_start(service.clone(), Some(user.clone()), Some(conv), &mut utpamh);
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &utpamh) returned {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    }

    retval = utpam_end(&mut utpamh, 0);
    if retval != PAM_SUCCESS {
        println!("utpam_end (pamh, 0) returned {}", retval);
        return 1;
    }

    0
}

#[test]
fn test_utpam_end() {
    assert_eq!(tst_utpam_end(), 0);
}
