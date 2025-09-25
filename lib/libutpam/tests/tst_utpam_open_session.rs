/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use utpam::common::PAM_SUCCESS;
use utpam::utpam_session::utpam_open_session;
fn tst_utpam_open_session() -> u8 {
    let retval = utpam_open_session(&mut None, 0);

    if retval == PAM_SUCCESS {
        println!("utpam_open_session (NULL, 0) returned PAM_SUCCESS");
        1;
    }
    0
}

#[test]
fn test_utpam_open_session() {
    assert_eq!(tst_utpam_open_session(), 0);
}
