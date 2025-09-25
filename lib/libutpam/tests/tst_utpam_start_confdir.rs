/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use utpam::common::{UtpamConv, PAM_SUCCESS};
use utpam::utpam::UtpamHandle;
use utpam::utpam_end::utpam_end;
use utpam::utpam_start::utpam_start_confdir;

fn tst_utpam_start_confdir() -> u8 {
    let service = "confdir".to_string();
    let xservice = "nonexistent-service".to_string();
    let user = "root".to_string();
    let xconfdir = "/nonexistent-confdir".to_string();
    let conv = Rc::new(UtpamConv {
        conv: None,
        appdata_ptr: None,
    });

    let confdir = match env::var("srcdir") {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {}", e);
            return 1;
        }
    };

    let mut utpamh: Option<Box<UtpamHandle>> = None;

    /* 1: check with valid arguments */
    let mut retval = utpam_start_confdir(
        service.clone(),
        Some(user.clone()),
        Some(conv.clone()),
        PathBuf::from(confdir.clone()),
        &mut utpamh,
    );

    if retval != PAM_SUCCESS {
        println!(
            "utpam_start_confdir ({}, {}, &conv, {}, &pamh) returned {}",
            service.clone(),
            user.clone(),
            confdir.clone(),
            retval
        );
        return 1;
    } else if utpamh.is_none() {
        println!(
            "utpam_start_confdir ({}, {}, &conv, {}, &pamh) returned NULL for pamh",
            service.clone(),
            user.clone(),
            confdir.clone()
        );
        return 1;
    }

    utpam_end(&mut utpamh, retval.into());

    retval = utpam_start_confdir(
        xservice.clone(),
        Some(user.clone()),
        Some(conv.clone()),
        PathBuf::from(confdir.clone()),
        &mut utpamh,
    );
    if retval == PAM_SUCCESS {
        println!(
            "utpam_start_confdir ({}, {}, &conv, {}, &pamh)  incorrectly succeeded",
            xservice.clone(),
            user.clone(),
            confdir.clone()
        );
        return 1;
    }
    utpam_end(&mut utpamh, retval.into());

    retval = utpam_start_confdir(
        service.clone(),
        Some(user.clone()),
        Some(conv.clone()),
        PathBuf::from(xconfdir.clone()),
        &mut utpamh,
    );

    if retval == PAM_SUCCESS {
        println!(
            "utpam_start_confdir ({}, {}, &conv, {}, &pamh)  incorrectly succeeded",
            service.clone(),
            user.clone(),
            xconfdir.clone()
        );
        return 1;
    }

    utpam_end(&mut utpamh, retval.into());
    0
}

#[test]
fn test_utpam_start_confdir() {
    assert_eq!(tst_utpam_start_confdir(), 0);
}
