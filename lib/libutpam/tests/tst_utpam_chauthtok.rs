/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::PAM_SUCCESS;
use utpam::utpam_password::utpam_chauthtok;
fn tst_utpam_chauthtok() -> u8 {
    let retval = utpam_chauthtok(&mut None, 0);

    if retval == PAM_SUCCESS {
        println!("utpam_chauthtok (NULL, 0) returned PAM_SUCCESS");
        1;
    }
    0
}

#[test]
fn test_utpam_chauthtok() {
    assert_eq!(tst_utpam_chauthtok(), 0);
}
