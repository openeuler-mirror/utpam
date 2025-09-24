/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::PAM_SUCCESS;
use utpam::utpam_auth::utpam_authenticate;
fn tst_utpam_authenticate() -> u8 {
    let retval = utpam_authenticate(&mut None, 0);

    if retval == PAM_SUCCESS {
        println!("utpam_authenticate (NULL, 0) returned PAM_SUCCESS");
        1;
    }
    0
}

#[test]
fn test_utpam_authenticate() {
    assert_eq!(tst_utpam_authenticate(), 0);
}
