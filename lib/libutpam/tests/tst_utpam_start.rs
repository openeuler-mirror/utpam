/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::{UtpamConv, PAM_SUCCESS};
use utpam::utpam::UtpamHandle;
use utpam::utpam_end::utpam_end;
use utpam::utpam_start::utpam_start;
fn tst_utpam_start() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = UtpamConv {
        conv: None,
        appdata_ptr: None,
    };
    let mut utpamh: Option<Box<UtpamHandle>> = None;

    /* 1: check with valid arguments */
    let mut retval = utpam_start(
        service.clone(),
        Some(user.clone()),
        Some(conv.clone()),
        &mut utpamh,
    );

    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &utpamh) returned {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    } else if utpamh.is_none() {
        println!(
            "utpam_start ({}, {}, &utpamh) returned NULL for utpamh {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    }

    utpam_end(&mut utpamh, retval.into());

    /* 2: check with NULL for service */
    retval = utpam_start(
        String::default(),
        Some(user.clone()),
        Some(conv.clone()),
        &mut utpamh,
    );
    if retval == PAM_SUCCESS {
        println!(
            "utpam_start (NULL, {}, &conv, &pamh) returned {}",
            user.clone(),
            retval
        );
        return 1;
    }

    /* 3: check with NULL for user */
    retval = utpam_start(service.clone(), None, Some(conv.clone()), &mut utpamh);
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, NULL, &conv, &pamh) returned {}",
            service.clone(),
            retval
        );
        return 1;
    }

    utpam_end(&mut utpamh, retval.into());

    /* 4: check with NULL for conv */
    retval = utpam_start(service.clone(), Some(user.clone()), None, &mut utpamh);
    if retval == PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, NULL, &pamh) returned {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    }

    0
}

#[test]
fn test_utpam_start() {
    assert_eq!(tst_utpam_start(), 0);
}
