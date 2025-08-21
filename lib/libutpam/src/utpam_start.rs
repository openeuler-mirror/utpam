/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut)]
#![allow(unused_variables)]
use crate::common::*;
use crate::utpam::*;
pub fn utpam_stat(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    let confdir = None;
    utpam_start_internal(service_name, user, utpam_conversation, confdir, utpamh)
}

pub fn utpam_stat_confdir(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: Option<String>,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    utpam_start_internal(service_name, user, utpam_conversation, confdir, utpamh)
}

fn utpam_start_internal(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: Option<String>,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    //let users: Option<String> = Some(String::from("bob"));
    let default_value = String::from("default_user");

    let mut pamh = UtpamHandle::new(service_name, utpam_conversation, user);

    //待开发
    PAM_SUCCESS
}
