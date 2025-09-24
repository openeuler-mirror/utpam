/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::PAM_SUCCESS;
use utpam::utpam_auth::utpam_setcred;
fn tst_utpam_setcred() -> u8 {
    let retval = utpam_setcred(&mut None, 0);

    if retval == PAM_SUCCESS {
        println!("utpam_setcred (NULL, 0) returned PAM_SUCCESS");
        1;
    }
    0
}

#[test]
fn test_utpam_setcred() {
    assert_eq!(tst_utpam_setcred(), 0);
}
