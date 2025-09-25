/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::any::Any;
use std::rc::Rc;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_item::utpam_get_item;
use utpam::utpam_start::utpam_start;

#[derive(Debug)]
struct Mappget {
    map_type: i8,
    string: &'static str,
    expected: u8,
}

// 定义全局常量
const ITEMS: &[Mappget] = &[
    Mappget {
        map_type: PAM_SERVICE,
        string: "PAM_SERVICE",
        expected: PAM_SUCCESS,
    },
    Mappget {
        map_type: PAM_USER,
        string: "PAM_USER",
        expected: 0,
    },
    Mappget {
        map_type: PAM_TTY,
        string: "PAM_TTY",
        expected: 0,
    },
    Mappget {
        map_type: PAM_RHOST,
        string: "PAM_RHOST",
        expected: 0,
    },
    Mappget {
        map_type: PAM_CONV,
        string: "PAM_CONV",
        expected: 0,
    },
    Mappget {
        map_type: PAM_AUTHTOK,
        string: "PAM_AUTHTOK",
        expected: PAM_BAD_ITEM,
    },
    Mappget {
        map_type: PAM_OLDAUTHTOK,
        string: "PAM_OLDAUTHTOK",
        expected: PAM_BAD_ITEM,
    },
    Mappget {
        map_type: PAM_RUSER,
        string: "PAM_RUSER",
        expected: 0,
    },
    Mappget {
        map_type: PAM_USER_PROMPT,
        string: "PAM_USER_PROMPT",
        expected: 0,
    },
    Mappget {
        map_type: PAM_FAIL_DELAY,
        string: "PAM_FAIL_DELAY",
        expected: 0,
    },
    Mappget {
        map_type: PAM_AUTHTOK_TYPE,
        string: "PAM_AUTHTOK_TYPE",
        expected: 0,
    },
];

fn tst_utpam_get_item() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = Rc::new(UtpamConv {
        conv: None,
        appdata_ptr: None,
    });
    let mut utpamh: Option<Box<UtpamHandle>> = None;
    let mut retval: u8;

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
    let mut value: Box<dyn Any> = Box::new(());
    let mut utpamh = utpamh.unwrap();
    for item in ITEMS.iter() {
        retval = utpam_get_item(&mut utpamh, item.map_type, &mut value);

        if retval != item.expected {
            println!(
                "utpam_get_item failed to get value for {}. Returned {}",
                item.string, retval
            );
            return 1;
        }
    }

    retval = utpam_get_item(&mut utpamh, -1, &mut value);

    if retval != PAM_BAD_ITEM {
        println!(
            "utpam_get_item returned {} when expecting PAM_BAD_ITEM",
            retval
        );
        return 1;
    }

    0
}

#[test]
fn test_utpam_get_item() {
    assert_eq!(tst_utpam_get_item(), 0);
}
