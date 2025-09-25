/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::rc::Rc;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_data::utpam_get_data;
use utpam::utpam_start::utpam_start;

#[test]
fn test_utpam_get_data() {
    let mut constdataptr = None;
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = Rc::new(UtpamConv {
        conv: None,
        appdata_ptr: None,
    });
    let mut retval: u8;

    retval = utpam_get_data(&mut None, Some("tst-utpam_get_data-1"), &mut constdataptr);
    assert_ne!(PAM_SUCCESS, retval); //判断是否不相等

    let mut utpamh: Option<Box<UtpamHandle>> = None;
    retval = utpam_start(service.clone(), Some(user.clone()), Some(conv), &mut utpamh);
    assert_eq!(PAM_SUCCESS, retval); //判断是否相等

    retval = utpam_get_data(&mut utpamh, Some("tst-utpam_get_data-2"), &mut constdataptr);
    assert_eq!(PAM_SUCCESS, retval);
}
