/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::any::Any;
use utpam::common::*;
use utpam::utpam::UtpamHandle;
use utpam::utpam_item::utpam_get_item;
use utpam::utpam_item::utpam_set_item;
use utpam::utpam_start::utpam_start;

#[derive(Debug)]
struct Mapping {
    map_type: i8,
    string: &'static str,
    expected: u8,
    new_value: &'static str,
}

// 定义全局常量
const ITEMS: &[Mapping] = &[
    Mapping {
        map_type: PAM_SERVICE,
        string: "PAM_SERVICE",
        expected: PAM_SUCCESS,
        new_value: "logout",
    },
    Mapping {
        map_type: PAM_USER,
        string: "PAM_USER",
        expected: PAM_SUCCESS,
        new_value: "noroot",
    },
    Mapping {
        map_type: PAM_TTY,
        string: "PAM_TTY",
        expected: PAM_SUCCESS,
        new_value: "TTyX",
    },
    Mapping {
        map_type: PAM_RHOST,
        string: "PAM_RHOST",
        expected: PAM_SUCCESS,
        new_value: "remote",
    },
    Mapping {
        map_type: PAM_AUTHTOK,
        string: "PAM_AUTHTOK",
        expected: PAM_BAD_ITEM,
        new_value: "none",
    },
    Mapping {
        map_type: PAM_OLDAUTHTOK,
        string: "PAM_OLDAUTHTOK",
        expected: PAM_BAD_ITEM,
        new_value: "none",
    },
    Mapping {
        map_type: PAM_RUSER,
        string: "PAM_RUSER",
        expected: PAM_SUCCESS,
        new_value: "noroot",
    },
    Mapping {
        map_type: PAM_USER_PROMPT,
        string: "PAM_USER_PROMPT",
        expected: PAM_SUCCESS,
        new_value: "your name: ",
    },
    Mapping {
        map_type: PAM_FAIL_DELAY,
        string: "PAM_FAIL_DELAY",
        expected: PAM_SUCCESS,
        new_value: "4000",
    },
    Mapping {
        map_type: PAM_AUTHTOK_TYPE,
        string: "PAM_AUTHTOK_TYPE",
        expected: PAM_SUCCESS,
        new_value: "U**X",
    },
];

fn tst_utpam_set_item() -> u8 {
    let service = "dummy".to_string();
    let user = "root".to_string();
    let conv = UtpamConv {
        conv: None,
        appdata_ptr: None,
    };
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

    let mut utpamh = utpamh.unwrap();
    retval = utpam_set_item(&mut utpamh, -1, Some(Box::new("dummy")));
    if retval != PAM_BAD_ITEM {
        println!(
            "utpam_set_item returned {} when expecting PAM_BAD_ITEM",
            retval
        );
        return 1;
    }

    retval = utpam_set_item(&mut utpamh, PAM_CONV, None);
    if retval != PAM_PERM_DENIED {
        println!("utpam_set_item (pamh, PAM_CONV, NULL) returned {}", retval);
        return 1;
    }

    for item in ITEMS.iter() {
        retval = utpam_set_item(&mut utpamh, item.map_type, Some(Box::new(item.new_value)));
        if retval != item.expected {
            println!(
                "utpam_set_item failed to set value for {}. Returned {}",
                item.string, retval
            );
            return 1;
        } else if item.expected == PAM_SUCCESS {
            let mut value: Box<dyn Any> = Box::new(());
            retval = utpam_get_item(&mut utpamh, item.map_type, &mut value);

            if retval != PAM_SUCCESS {
                println!(
                    "utpam_get_item was not able to fetch changed value: {}",
                    retval
                );
                return 1;
            }
            if let Some(val) = value.downcast_ref::<String>() {
                if item.new_value == val {
                    println!(
                        "utpam_get_item got wrong value: expected: {} got: {}",
                        item.new_value, val
                    );
                    return 1;
                }
            }
        }
    }

    0
}

#[test]
fn test_utpam_set_item() {
    assert_eq!(tst_utpam_set_item(), 0);
}
