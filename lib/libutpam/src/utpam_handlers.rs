/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unused_variables, unused_assignments)]

use crate::common::{PAM_ABORT, PAM_IGNORE, PAM_NEW_AUTHTOK_REQD, PAM_RETURN_VALUES, PAM_SUCCESS};
use crate::utpam::*;
use crate::utpam_misc::{utpam_parse_control, utpam_set_default_control, utpam_tokenize};
use utpam_internal::utpam_line::{utpam_line_assemble, UtpamLineBuffer};

use std::fs::{metadata, File, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;

const PAM_T_ANY: i32 = 0;
const PAM_T_AUTH: i32 = 1;
const PAM_T_SESS: i32 = 2;
const PAM_T_ACCT: i32 = 4;
const PAM_T_PASS: i32 = 8;

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
                Some(utpamh.service_name.clone()),
                0,
                0,
                0,
                false,
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
                println!("打开文件失败");
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
                    println!("打开文件失败");
                    return PAM_ABORT;
                }
            }
        } else {
            println!("找不到文件或目录: {:?}", path_buf);
        }
    }

    PAM_ABORT
}

//加载并解析指定的配置文件
fn utpam_load_conf_file(
    utpamh: &mut Box<UtpamHandle>,
    config_name: Option<String>,
    service: Option<String>,
    module_type: i32,
    include_level: i32,
    stack_level: i32,
    not_other: bool,
) -> i32 {
    let mut file: Option<File> = None;
    let mut path: Option<PathBuf> = None;
    let mut retval: i32 = PAM_ABORT;

    //检查是否超过了最大允许的嵌套层次
    if include_level >= PAM_SUBSTACK_MAX_LEVEL {
        //日记记录
        println!("maximum level of inclusions reached");
        return PAM_ABORT;
    }

    //检查配置文件名
    let config_name = match config_name {
        Some(name) => name,
        None => {
            //日记记录
            println!("no config file supplied");
            return PAM_ABORT;
        }
    };

    //打开配置文件
    if utpam_open_config_file(utpamh, config_name.clone(), &mut path, &mut file) == PAM_SUCCESS {
        //解析配置文件
        retval = utpam_parse_config_file(
            utpamh,
            file.unwrap(),
            service,
            module_type,
            include_level,
            stack_level,
            not_other,
        );
        if retval != PAM_SUCCESS {
            //日记记录
            println!("utpam_load_conf_file: error reading {:?}", path)
        }
    } else {
        //日记记录
        println!(
            "utpam_load_conf_file: unable to open config for {}",
            config_name.clone()
        )
    }
    retval
}

//解析配置文件
fn utpam_parse_config_file(
    utpamh: &mut Box<UtpamHandle>,
    file: File,
    known_service: Option<String>,
    requested_module_type: i32,
    include_level: i32,
    stack_level: i32,
    not_other: bool,
) -> i32 {
    let mut f = BufReader::new(file);
    let mut buffer = UtpamLineBuffer::default();
    let repl = String::from(" ");
    let mut tok = String::from("");
    let mut handler_type = PAM_HT_MODULE;
    let mut module_type;
    let mut actions: Vec<i32> = vec![0; PAM_RETURN_VALUES];

    //逐行处理配置文件内容
    let mut x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    while x > 0 {
        let mut mod_path: Option<String> = None;
        let mut nexttok: Option<String> = None;
        let mut buf = Some(buffer.assembled.as_str());
        //判断是否提供服务名称
        let this_service = match known_service {
            Some(ref s) => {
                nexttok = Some(s.clone());
                s.clone()
            }
            None => match utpam_tokenize(None, &mut buf) {
                Some(s) => {
                    tok = s;
                    tok.clone()
                }
                None => String::from(""),
            },
        };

        let other = if not_other {
            false
        } else {
            this_service.eq_ignore_ascii_case(UTPAM_DEFAULT_SERVICE)
        };

        let accspt = this_service.eq_ignore_ascii_case(&utpamh.service_name.clone());

        if !accspt || other {
            let mut pam_include = 0;
            let mut substack = 0;

            match utpam_tokenize(None, &mut buf) {
                Some(mut tok) => {
                    if tok.starts_with('-') {
                        handler_type = PAM_HT_SILENT_MODULE;
                        tok = tok.strip_prefix('-').unwrap().to_string();
                    }
                    if tok.eq_ignore_ascii_case("auth") {
                        module_type = PAM_T_AUTH;
                    } else if tok.eq_ignore_ascii_case("session") {
                        module_type = PAM_T_SESS;
                    } else if tok.eq_ignore_ascii_case("account") {
                        module_type = PAM_T_ACCT;
                    } else if tok.eq_ignore_ascii_case("password") {
                        module_type = PAM_T_PASS;
                    } else {
                        //无效的类型，日记记录
                        module_type = if requested_module_type != PAM_T_ANY {
                            requested_module_type
                        } else {
                            PAM_T_AUTH
                        };

                        handler_type = PAM_HT_MUST_FAIL;
                    }
                }
                None => {
                    //模块类型不存在，日记记录
                    module_type = if requested_module_type != PAM_T_ANY {
                        requested_module_type
                    } else {
                        PAM_T_AUTH
                    };

                    handler_type = PAM_HT_MUST_FAIL;
                }
            };
            if requested_module_type != PAM_T_ANY && module_type != requested_module_type {
                //日志记录
                continue;
            }

            for item in actions.iter_mut().take(PAM_RETURN_VALUES) {
                *item = PAM_ACTION_UNDEF;
            }

            //读取控制标志
            match utpam_tokenize(None, &mut buf) {
                Some(tok) => {
                    //将tok转换为小写字母后进行匹配
                    match tok.to_ascii_lowercase().as_str() {
                        "required" => {
                            actions[PAM_SUCCESS as usize] = PAM_ACTION_OK;
                            actions[PAM_NEW_AUTHTOK_REQD as usize] = PAM_ACTION_OK;
                            actions[PAM_IGNORE as usize] = PAM_ACTION_IGNORE;
                            utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                        }
                        "requisite" => {
                            actions[PAM_SUCCESS as usize] = PAM_ACTION_OK;
                            actions[PAM_NEW_AUTHTOK_REQD as usize] = PAM_ACTION_OK;
                            actions[PAM_IGNORE as usize] = PAM_ACTION_IGNORE;
                            utpam_set_default_control(&mut actions, PAM_ACTION_DIE);
                        }
                        "optional" => {
                            actions[PAM_SUCCESS as usize] = PAM_ACTION_OK;
                            actions[PAM_NEW_AUTHTOK_REQD as usize] = PAM_ACTION_OK;
                            actions[PAM_IGNORE as usize] = PAM_ACTION_IGNORE;
                            utpam_set_default_control(&mut actions, PAM_ACTION_IGNORE);
                        }
                        "sufficient" => {
                            actions[PAM_SUCCESS as usize] = PAM_ACTION_DONE;
                            actions[PAM_NEW_AUTHTOK_REQD as usize] = PAM_ACTION_DONE;
                            actions[PAM_IGNORE as usize] = PAM_ACTION_IGNORE;
                            utpam_set_default_control(&mut actions, PAM_ACTION_IGNORE);
                        }
                        "include" => {
                            pam_include = 1;
                            substack = 0;
                        }
                        "substack" => {
                            pam_include = 1;
                            substack = 1;
                        }
                        _ => {
                            //无效的控制标志，日志记录
                            utpam_parse_control(&mut actions, &tok);
                            utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                        }
                    }
                }
                None => {
                    //日志记录
                    utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                    handler_type = PAM_HT_MUST_FAIL;
                }
            }

            //读取调用路径或者认证模块
            match utpam_tokenize(None, &mut buf) {
                Some(tok) => {
                    if pam_include == 1 {
                        if substack == 1 {
                            //pam_add_handler函数  --待开发
                        }
                        if utpam_load_conf_file(
                            utpamh,
                            Some(tok),
                            Some(this_service),
                            module_type,
                            include_level + 1,
                            stack_level + substack,
                            not_other,
                        ) == PAM_SUCCESS
                        {
                            println!("include success");
                            //continue;
                        }

                        utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                        mod_path = None;
                        handler_type = PAM_HT_MUST_FAIL;
                        nexttok = None;
                    } else {
                        mod_path = Some(tok);
                    }
                }
                None => {
                    //没有给出模块名称
                    // 日志记录
                    mod_path = None;
                    handler_type = PAM_HT_MUST_FAIL;
                }
            }
        }
        //更新循环
        x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    }
    0
}
