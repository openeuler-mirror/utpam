/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::{PAM_DATA_REPLACE, PAM_NO_MODULE_DATA, PAM_SUCCESS, PAM_SYSTEM_ERR};
use crate::utpam::*;
use crate::{IF_NO_UTPAMH, UTPAM_FROM_APP};
use std::any::Any;
use std::rc::Rc;
/// 查找UtpamData结构体里的特定数据
pub fn utpam_locate_data<'a>(utpamh: &'a UtpamHandle, name: &'a str) -> Option<&'a UtpamData> {
    let mut current = utpamh.data.as_ref();
    while let Some(data) = current {
        if data.name == name {
            return Some(data);
        }
        current = data.next.as_ref();
    }
    None
}

/// 设置指定模块的数据
pub fn utpam_set_data(
    utpamh: &mut Option<Box<UtpamHandle>>,
    module_data_name: Option<&str>,
    data: Option<Rc<dyn Any>>,
    cleanup: Option<CleanupFn>,
) -> i32 {
    let mut data_entry = UtpamData {
        name: String::default(),
        data: None,
        cleanup: None,
        next: None,
    };

    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_APP!(utpamh) {
        println!("called from application!?");
        return PAM_SYSTEM_ERR;
    }

    let module_data_name = match module_data_name {
        Some(name) => name,
        None => {
            println!("called with NULL as module_data_name");
            return PAM_SYSTEM_ERR;
        }
    };

    // 查找数据
    let mut utpamdata = utpam_locate_data(utpamh, module_data_name).cloned();
    match utpamdata {
        Some(ref mut entry) => {
            // 如果存在cleanup函数，则调用
            if let Some(ref mut cleanup) = entry.cleanup {
                cleanup(utpamh, entry.data.clone(), PAM_DATA_REPLACE | PAM_SUCCESS)
            }
        }
        None => {
            // 如果不存在，则创建一个新的数据条目
            data_entry.next = utpamh.data.take();
            utpamh.data = Some(Box::new(data_entry.clone()));
            data_entry.name = module_data_name.to_string();
        }
    }

    // 设置数据
    data_entry.data = data;
    data_entry.cleanup = cleanup;

    PAM_SUCCESS
}

// 获取数据
pub fn utpam_get_data(
    utpamh: &mut Option<Box<UtpamHandle>>,
    module_data_name: Option<&str>,
    datap: &mut Option<Rc<dyn Any>>,
) -> i32 {
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_SYSTEM_ERR);

    if UTPAM_FROM_APP!(utpamh) {
        println!("called from application!?");
        return PAM_SYSTEM_ERR;
    }

    let module_data_name = match module_data_name {
        Some(name) => name,
        None => {
            println!("called with NULL as module_data_name");
            return PAM_SYSTEM_ERR;
        }
    };

    if let Some(data_entry) = utpam_locate_data(utpamh, module_data_name) {
        *datap = data_entry.data.clone();
        return PAM_SUCCESS;
    }

    PAM_NO_MODULE_DATA
}

// 清理数据
pub fn utpam_free_data(utpamh: &mut Box<UtpamHandle>, status: i32) {
    let mut current = utpamh.data.take();
    while let Some(mut data) = current {
        current = data.next.take();
        if let Some(ref mut cleanup) = data.cleanup {
            cleanup(utpamh, data.data.clone(), status);
        }
    }
}
