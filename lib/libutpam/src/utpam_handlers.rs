/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::too_many_arguments)]

use crate::common::*;
use crate::pam_syslog;
use crate::utpam::*;
use crate::utpam_strerror::pam_strerror;
use crate::utpam_syslog::*;

use crate::utpam_dynamic::{utpam_dlopen, utpam_dlsym};
use crate::utpam_misc::{
    utpam_mkargv, utpam_parse_control, utpam_set_default_control, utpam_tokenize,
};
use std::env::consts::ARCH;
use std::path::Path;
use utpam_internal::utpam_line::{utpam_line_assemble, UtpamLineBuffer};

use std::cell::RefCell;
use std::fs::{metadata, File, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;

const PAM_T_ANY: i32 = 0;
const PAM_T_AUTH: i32 = 1;
const PAM_T_SESS: i32 = 2;
const PAM_T_ACCT: i32 = 4;
const PAM_T_PASS: i32 = 8;

const UNKNOWN_MODULE: &str = "unknown module";
const DEFAULT_MODULE_PATH: &str = "/lib64/security";

//初始化UtpamHandle结构体字段
pub fn utpam_init_handlers(utpamh: &mut Box<UtpamHandle>) -> i32 {
    let mut retval: i32;

    //如果所有内容都已加载，则立即返回
    if utpamh.handlers.handlers_loaded != 0 {
        return PAM_SUCCESS;
    }

    if utpamh.service_name.is_empty() {
        return PAM_BAD_ITEM;
    }

    //PAM_LOCKING-检查子系统是否锁定：条件编译（最后处理）

    //解析配置文件
    let mut path: Option<PathBuf> = None;
    let mut file: Option<File> = None;

    //检查配置目录是否存在
    if utpamh.confdir.exists()
        || check_directory(UTPAM_CONFIG_D)
        || check_directory(UTPAM_CONFIG_DIST_D)
    {
        let mut read_something = 0;
        //打开配置文件并解析
        if utpam_open_config_file(utpamh, utpamh.service_name.clone(), &mut path, &mut file)
            == PAM_SUCCESS
        {
            //解析配置文件
            retval = utpam_parse_config_file(
                utpamh,
                file.as_mut().unwrap(),
                Some(utpamh.service_name.clone()),
                PAM_T_ANY,
                0,
                0,
                false,
            );

            if retval != PAM_SUCCESS {
                let path = if let Some(s) = &path {
                    s.to_str().unwrap()
                } else {
                    "unknown file"
                };
                pam_syslog!(
                    &utpamh,
                    LOG_ERR,
                    "utpam_init_handlers: error reading {}",
                    path
                );
                let err = pam_strerror(utpamh, LOG_ERR as i64);
                pam_syslog!(&utpamh, LOG_ERR, "utpam_init_handlers [{}]", err);
            } else {
                read_something = 1;
            }
        } else {
            println!(
                "unable to open configuration for: {:?}",
                utpamh.service_name
            );
            retval = PAM_SUCCESS;
        }

        if retval == PAM_SUCCESS {
            if utpam_open_config_file(
                utpamh,
                UTPAM_DEFAULT_SERVICE.to_string(),
                &mut path,
                &mut file,
            ) == PAM_SUCCESS
            {
                retval = utpam_parse_config_file(
                    utpamh,
                    file.as_mut().unwrap(),
                    Some(UTPAM_DEFAULT_SERVICE.to_string()),
                    PAM_T_ANY,
                    0,
                    0,
                    false,
                );
                if retval != PAM_SUCCESS {
                    let path = if let Some(s) = &path {
                        s.to_str().unwrap()
                    } else {
                        "unknown file"
                    };
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "utpam_init_handlers: error reading {}",
                        path
                    );
                    let err = pam_strerror(utpamh, LOG_ERR as i64);
                    pam_syslog!(&utpamh, LOG_ERR, "utpam_init_handlers: [{}]", err);
                } else {
                    read_something = 1;
                }
            } else {
                pam_syslog!(
                    &utpamh,
                    LOG_ERR,
                    "utpam_init_handlers: no default config {}",
                    UTPAM_DEFAULT_SERVICE
                );
            }
            if read_something == 0 {
                retval = PAM_ABORT;
            }
        }
    } else {
        let path = Path::new(UTPAM_CONFIG);
        match File::open(path) {
            Ok(ref mut file) => {
                retval = utpam_parse_config_file(utpamh, file, None, PAM_T_ANY, 0, 0, false);
            }
            Err(_) => {
                pam_syslog!(
                    &utpamh,
                    LOG_ERR,
                    "utpam_init_handlers: error reading {}",
                    UTPAM_CONFIG
                );
                return PAM_ABORT;
            }
        };
    }

    if retval != PAM_SUCCESS {
        pam_syslog!(&utpamh, LOG_ERR, "error reading PAM configuration file",);
        return PAM_ABORT;
    }

    utpamh.handlers.handlers_loaded = 1;

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
            Err(err) => {
                pam_syslog!(&utpamh, LOG_ERR, "{}", err);
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
                Err(err) => {
                    pam_syslog!(&utpamh, LOG_ERR, "{}", err);
                    return PAM_ABORT;
                }
            }
        } else {
            println!("File or directory not found: {:?}", path_buf);
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
            file.as_mut().unwrap(),
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
    file: &mut File,
    known_service: Option<String>,
    requested_module_type: i32,
    include_level: i32,
    stack_level: i32,
    not_other: bool,
) -> i32 {
    let mut f = BufReader::new(file);
    let mut buffer = UtpamLineBuffer::default();
    let repl = String::from(" ");
    let mut tok;
    let mut handler_type = PAM_HT_MODULE;
    let mut module_type;
    let mut actions: Vec<i32> = vec![0; PAM_RETURN_VALUES];

    //逐行处理配置文件内容
    let mut x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    while x > 0 {
        let mut mod_path;
        //let mut nexttok: Option<String> = None;
        let mut buf = Some(buffer.assembled.as_str());
        let mut res;
        let mut argc = 0;
        let mut argv: Vec<String> = vec![];

        //判断是否提供服务名称
        let this_service = match known_service {
            Some(ref s) => {
                //nexttok = Some(s.clone());
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
                        pam_syslog!(
                            &utpamh,
                            LOG_ERR,
                            "({}) illegal module type: {}",
                            this_service,
                            tok
                        );
                        module_type = if requested_module_type != PAM_T_ANY {
                            requested_module_type
                        } else {
                            PAM_T_AUTH
                        };

                        handler_type = PAM_HT_MUST_FAIL;
                    }
                }
                None => {
                    pam_syslog!(&utpamh, LOG_ERR, "({}) empty module type", this_service);
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
                            println!("will need to parse {}", tok);
                            utpam_parse_control(&mut actions, &tok);
                            utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                        }
                    }
                }
                None => {
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "({}) no control flag supplied",
                        this_service
                    );
                    utpam_set_default_control(&mut actions, PAM_ACTION_BAD);
                    handler_type = PAM_HT_MUST_FAIL;
                }
            }

            //读取调用路径或者认证模块
            match utpam_tokenize(None, &mut buf) {
                Some(tok) => {
                    if pam_include == 1 {
                        if substack == 1 {
                            res = utpam_add_handler(
                                utpamh,
                                PAM_HT_SUBSTACK,
                                other,
                                stack_level,
                                module_type,
                                &mut actions,
                                &Some(tok.clone()),
                                argc,
                                &argv,
                            );
                            if res != PAM_SUCCESS {
                                pam_syslog!(&utpamh, LOG_ERR, "error adding substack {}", tok);
                                return PAM_ABORT;
                            }
                        }
                        if utpam_load_conf_file(
                            utpamh,
                            Some(tok.clone()),
                            Some(this_service.clone()),
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
                        //nexttok = None;
                    } else {
                        mod_path = Some(tok);
                    }
                }
                None => {
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "({}) no module name supplied",
                        this_service
                    );
                    mod_path = None;
                    handler_type = PAM_HT_MUST_FAIL;
                }
            }

            if let Some(buf) = buf {
                if utpam_mkargv(buf, &mut argv, &mut argc) == 0 {
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "({}) argument vector allocation failed",
                        this_service
                    );
                    mod_path = None;
                    handler_type = PAM_HT_MUST_FAIL;
                }
            }

            res = utpam_add_handler(
                utpamh,
                handler_type,
                other,
                stack_level,
                module_type,
                &mut actions,
                &mod_path,
                argc,
                &argv,
            );
            if res != PAM_SUCCESS {
                let mod_path = match mod_path {
                    Some(path) => path,
                    None => "unknown file".to_string(),
                };
                pam_syslog!(&utpamh, LOG_ERR, "error loading {}", mod_path);
                return PAM_ABORT;
            }
        }
        //更新循环
        x = utpam_line_assemble(&mut f, &mut buffer, repl.clone());
    }

    if x < 0 {
        PAM_ABORT
    } else {
        PAM_SUCCESS
    }
}

///从模块路径中提取模块名称，如果路径无效则返回 None
fn extract_modulename(mod_path: &str) -> Option<String> {
    //如果路径为空
    if mod_path.trim().is_empty() {
        return None;
    }
    let path = PathBuf::from(mod_path);
    let file_stem = path.file_stem(); // 获取文件名（不包括扩展名）

    let file_stem_str = match file_stem {
        Some(stem) => stem.to_str()?,
        None => return None,
    };

    //检查路径是否有效
    if file_stem_str.is_empty() || file_stem_str == "?" {
        return None;
    }

    Some(file_stem_str.into())
}

//加载模块
fn utpam_load_module(
    utpamh: &mut Vec<LoadedModule>,
    mod_path: String,
    handler_type: i32,
) -> Option<&LoadedModule> {
    let mods = utpamh;

    //遍历模块列表，检查是否存在mod_path模块。
    let x = mods.iter().position(|m| m.name == mod_path);

    match x {
        Some(index) => {
            //直接返回匹配到的模块
            Some(&mods[index])
        }
        None => {
            /*处理没有匹配到模块的情况*/

            //创建一个新的模块
            let mut newmod = LoadedModule {
                name: mod_path.clone(),
                moule_type: 0,
                dl_handle: None,
            };

            //加载模块
            match utpam_dlopen(mod_path.clone().to_string()) {
                Ok(handle) => newmod.dl_handle = Some(handle),
                Err(e) => {
                    println!("Error loading module: {}", e);

                    //处理带有占位符的路径字符串，并根据具体的架构替换占位符以加载正确的动态库
                    let isa_pos = mod_path.find("$ISA");
                    if isa_pos.is_some() {
                        let target_arch = match ARCH {
                            "x86_64" => "x86_64",
                            "aarch64" => "aarch64",
                            _ => panic!("Unsupported architecture"),
                        };
                        let real_mod_path = mod_path.replace("$ISA", target_arch);

                        //重新加载模块
                        match utpam_dlopen(real_mod_path) {
                            Ok(handle) => {
                                newmod.moule_type = PAM_MT_DYNAMIC_MOD;
                                newmod.dl_handle = Some(handle);
                            }
                            Err(e) => {
                                newmod.dl_handle = None;
                                newmod.moule_type = PAM_MT_FAULTY_MOD;

                                if handler_type != PAM_HT_SILENT_MODULE {
                                    println!("{}", e);
                                }
                            }
                        };
                    } else {
                        println!("{}", e);
                    }
                }
            };

            //添加到模块列表
            let index = mods.len();
            mods.push(newmod);

            //返回新模块
            Some(&mods[index])
        }
    }
}

//添加处理程序
fn utpam_add_handler(
    utpamh: &mut Box<UtpamHandle>,
    handler_type: i32,
    other: bool,
    stack_level: i32,
    module_type: i32,
    actions: &mut [i32],
    mod_path: &Option<String>,
    argc: i32,
    argv: &[String],
) -> i32 {
    let mut load_module = None;

    //let mut mod_type: i32 = PAM_MT_FAULTY_MOD;
    let mod_type;
    let unknown_module = UNKNOWN_MODULE.to_string();

    //处理模块路径
    let mod_path = match mod_path {
        Some(s) => s,
        None => &unknown_module,
    };

    //根据模块路径获取模块类型
    if handler_type == PAM_HT_MODULE || handler_type == PAM_HT_SILENT_MODULE {
        let new_path = PathBuf::from(DEFAULT_MODULE_PATH).join(mod_path);
        if mod_path.starts_with('/') {
            load_module = utpam_load_module(
                &mut utpamh.handlers.module,
                mod_path.to_string(),
                handler_type,
            );
        } else if new_path.exists() {
            load_module = utpam_load_module(
                &mut utpamh.handlers.module,
                new_path.to_string_lossy().to_string(),
                handler_type,
            );
        } else {
            pam_syslog!(&utpamh, LOG_ERR, "cannot malloc full mod path",);
            return PAM_ABORT;
        }
    }

    //如果模块加载失败，则返回PAM_ABORT
    let load_module = match load_module {
        Some(m) => {
            mod_type = m.moule_type;
            m
        }
        None => return PAM_ABORT,
    };

    //决定使用哪个处理程序列表
    let the_handlers = if other {
        &mut utpamh.handlers.other
    } else {
        &mut utpamh.handlers.conf
    };

    //匹配处理程序类型
    let (handler_p, sym, handler_p2, sym2) = match module_type {
        PAM_T_AUTH => (
            &mut the_handlers.authenticate,
            "utpam_sm_authenticate",
            Some(&mut the_handlers.setcred),
            "utpam_sm_setcred",
        ),
        PAM_T_SESS => (
            &mut the_handlers.open_session,
            "utpam_sm_open_session",
            Some(&mut the_handlers.close_session),
            "utpam_sm_close_session",
        ),
        PAM_T_ACCT => (&mut the_handlers.acct_mgmt, "utpam_sm_acct_mgmt", None, ""),
        PAM_T_PASS => (&mut the_handlers.chauthtok, "utpam_sm_chauthtok", None, ""),
        _ => return PAM_ABORT,
    };

    if mod_type != PAM_MT_DYNAMIC_MOD && mod_type != PAM_MT_FAULTY_MOD {
        pam_syslog!(
            &utpamh,
            LOG_ERR,
            "internal error: module library type not known: {};{}",
            sym,
            mod_type
        );
        return PAM_ABORT;
    }

    let handle = match load_module.dl_handle {
        Some(ref s) => s,
        None => {
            println!("unable to dlopen");
            return PAM_ABORT;
        }
    };

    // 获取函数指针
    match utpam_dlsym(handle, sym.as_bytes()) {
        Ok(func) => {
            let path = match extract_modulename(mod_path) {
                Some(mod_name) => mod_name,
                None => return PAM_ABORT,
            };
            let handler = Handler {
                handler_type,
                func: Some(*func),
                actions: actions.to_owned(),
                cached_retval: Rc::new(RefCell::new(_PAM_INVALID_RETVAL)),
                argc,
                argv: argv.to_owned(),
                next: None,
                mod_name: path,
                stack_level,
                grantor: 0,
            };
            // 将新 Handler 插入链表末尾
            append_handler(handler_p, handler);
        }
        Err(_) => {
            pam_syslog!(&utpamh, LOG_ERR, "unable to resolve symbol: {}", sym);
            return PAM_ABORT;
        }
    }

    if !sym2.is_empty() {
        match utpam_dlsym(handle, sym2.as_bytes()) {
            Ok(func) => {
                // 如果存在第二个函数指针，则将其插入链表末尾
                if let Some(handler_p2) = handler_p2 {
                    let path = match extract_modulename(mod_path) {
                        Some(mod_name) => mod_name,
                        None => return PAM_ABORT,
                    };

                    let handler = Handler {
                        handler_type,
                        func: Some(*func),
                        actions: actions.to_owned(),
                        cached_retval: Rc::new(RefCell::new(_PAM_INVALID_RETVAL)),
                        argc,
                        argv: argv.to_owned(),
                        next: None,
                        mod_name: path,
                        stack_level,
                        grantor: 0,
                    };
                    append_handler(handler_p2, handler);
                }
            }
            Err(_) => {
                pam_syslog!(&utpamh, LOG_ERR, "unable to resolve symbol: {}", sym2);
                return PAM_ABORT;
            }
        }
    }

    PAM_SUCCESS
}

//在链表的末尾插入一个新的 Handler 节点
fn append_handler(handlers: &mut Option<Box<Handler>>, new_handler: Handler) {
    let mut current = handlers;

    //循环遍历链表，直到找到最后一个节点。如果当前节点的 next 是 None，则插入新的 Handler
    while let Some(ref mut node) = current {
        if node.next.is_none() {
            node.next = Some(Box::new(new_handler));
            return;
        }
        current = &mut node.next;
    }

    // 如果链表为空，直接插入新的 Handler
    if current.is_none() {
        *current = Some(Box::new(new_handler));
    }
}
