/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::PAM_SUCCESS;
use utpam::utpam_account::utpam_acct_mgmt;
fn tst_utpam_acct_mgmt() -> u8 {
    let retval = utpam_acct_mgmt(&mut None, 0);

    if retval == PAM_SUCCESS {
        println!("utpam_acct_mgmt (NULL, 0) returned PAM_SUCCESS");
        1;
    }
    0
}

#[test]
fn test_utpam_acct_mgmt() {
    assert_eq!(tst_utpam_acct_mgmt(), 0);
}
