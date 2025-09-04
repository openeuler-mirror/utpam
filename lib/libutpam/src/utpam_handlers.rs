/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::{PAM_ABORT, PAM_SUCCESS};
use crate::utpam::{UtpamHandle, UTPAM_CONFIG_D, UTPAM_CONFIG_DIST_D};
use utpam_internal::utpam_line::{utpam_line_assemble, UtpamLineBuffer};

use std::fs::{metadata, File, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;

pub fn utpam_init_handlers(utpamh: &mut Box<UtpamHandle>) -> i32 {
    //如果所有内容都已加载，则立即返回
    if utpamh.handlers.handlers_loaded != 0 {
        return PAM_SUCCESS;
    }

    //PAM_LOCKING-检查子系统是否锁定：条件编译（最后处理）

    //解析配置文件
    let mut path: Option<PathBuf> = None;
    let mut file: Option<File> = None;

    if utpamh.confdir.exists()
        || check_directory(UTPAM_CONFIG_D)
        || check_directory(UTPAM_CONFIG_DIST_D)
    {
        //从配置文件中获取配置目录

        if utpam_open_config_file(utpamh, utpamh.service_name.clone(), &mut path, &mut file)
            == PAM_SUCCESS
        {
            //解析配置文件(待开发)
            utpam_parse_config_file(
                utpamh,
                file.unwrap(),
                utpamh.service_name.clone(),
                0,
                0,
                0,
                None,
            );
        }
    }

    PAM_SUCCESS
}

///检查指定路径是否存在且是目录
fn check_directory(pamh_confdir: &str) -> bool {
    metadata(pamh_confdir).map_or(false, |md| md.file_type().is_dir())
}

///打开配置文件
fn utpam_open_config_file(
    utpamh: &mut Box<UtpamHandle>,
    service: String,
    path: &mut Option<PathBuf>,
    file: &mut Option<File>,
) -> i32 {
    let mut path_buf = PathBuf::new();

    let dirs = [UTPAM_CONFIG_D, UTPAM_CONFIG_DIST_D];

    //处理提供了绝对路径和配置目录的情况
    if service.starts_with('/') {
        path_buf = PathBuf::from(service.clone());
    } else if utpamh.confdir.exists() {
        path_buf = utpamh.confdir.join(service.clone());
    }
    if path_buf.is_file() {
        let fd = OpenOptions::new().read(true).open(&path_buf);
        match fd {
            Ok(f) => {
                *path = Some(path_buf);
                *file = Some(f);
                return PAM_SUCCESS;
            }
            Err(_) => {
                return PAM_ABORT;
            }
        }
    }

    //打开默认配置目录下的文件
    for dir in dirs {
        let dir = PathBuf::from(dir);
        path_buf = dir.join(service.clone());
        if path_buf.exists() {
            let fd = OpenOptions::new().read(true).open(&path_buf);
            match fd {
                Ok(f) => {
                    *path = Some(path_buf);
                    *file = Some(f);
                    return PAM_SUCCESS;
                }
                Err(_) => {
                    return PAM_ABORT;
                }
            }
        }
    }

    PAM_ABORT
}

//解析配置文件
fn utpam_parse_config_file(
    _utpamh: &mut Box<UtpamHandle>,
    file: File,
    _known_service: String,
    _requested_module_type: i32,
    _include_level: i32,
    _stack_level: i32,
    _not_other: Option<i32>,
) -> i32 {
    let mut f = BufReader::new(file);
    let mut buffer = UtpamLineBuffer::default();
    let repl = String::from(" ");

    //逐行处理配置文件内容
    let mut x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    while x > 0 {
        if !buffer.assembled.is_empty() {
            println!("{}", buffer.assembled);
        }

        //更新循环
        x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    }
    0
}
