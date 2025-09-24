/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_delay::utpam_fail_delay;
use utpam::utpam_start::utpam_start;

fn tst_utpam_fail_delay() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = UtpamConv {
        conv: None,
        appdata_ptr: None,
    };
    let mut retval: u8;
    let mut utpamh: Option<Box<UtpamHandle>> = None;
    retval = utpam_start(service.clone(), Some(user.clone()), Some(conv), &mut utpamh);
    if retval != PAM_SUCCESS {
        println!(
            "utpam_start ({}, {}, &conv, &pamh) returned {}",
            service.clone(),
            user.clone(),
            retval
        );
        return 1;
    }
    let mut utpam = utpamh.unwrap();
    retval = utpam_fail_delay(&mut utpam, 60);

    if retval != PAM_SUCCESS {
        println!("utpam_fail_delay (pamh, 60) returned {}", retval);
        return 1;
    }

    0
}

#[test]
fn test_utpam_fail_delay() {
    assert_eq!(tst_utpam_fail_delay(), 0);
}
