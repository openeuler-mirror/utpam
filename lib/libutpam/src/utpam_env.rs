/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::common::*;
use crate::utpam::{UtpamHandle, PAM_ENV_CHUNK};
use crate::utpam_misc::utpam_strdup;
use crate::utpam_syslog::*;
use crate::{pam_syslog, IF_NO_UTPAMH};

#[derive(Debug)]
pub struct UtpamEnviron {
    entries: i32,
    requested: usize,
    list: Vec<String>,
}

/// 创建环境变量
pub fn utpam_make_env(env: &mut Option<UtpamEnviron>) -> i32 {
    *env = Some(UtpamEnviron {
        entries: PAM_ENV_CHUNK,
        requested: 1,
        list: Vec::new(),
    });
    //_pam_dump_env(pamh); 待实现（输出调试信息）
    PAM_SUCCESS
}

/// 销毁环境变量
pub fn utpam_drop_env(utpamh: &mut Box<UtpamHandle>) {
    match utpamh.env {
        Some(ref mut env) => {
            // 清空环境变量列表
            for list in env.list.iter_mut() {
                list.clear()
            }

            env.entries = 0;
            env.requested = 0;
            env.list.clear(); //清空 Vec 本身
        }
        None => println!("no environment present in utpamh?"),
    }
}

/// 在环境变量列表中搜索给定变量名的位置
fn utpam_search_env(env: &UtpamEnviron, name_value: &str, length: usize) -> i32 {
    //从后向前遍历env数组
    for i in (0..env.requested).rev() {
        /* 获取索引为 i 的环境变量字符串:
         *  如果索引有效，检查字符串是否以 name_value 开头并且第 length 个字符为 '='，
         *  如果条件满足，返回当前索引 i；否则继续遍历
         */
        if env.list.get(i).map_or(false, |s| {
            s.starts_with(name_value) && s.chars().nth(length) == Some('=')
        }) {
            return i as i32; // Found it!
        }
    }
    -1
}

/// 添加、替换或删除环境变量
pub fn utpam_putenv(utpamh: &mut Option<Box<UtpamHandle>>, name_value: &str) -> i32 {
    let utpamh = IF_NO_UTPAMH!(utpamh, PAM_ABORT);

    if name_value.is_empty() {
        pam_syslog!(&utpamh, LOG_ERR, "utpam_putenv: no variable indicated",);
        return PAM_PERM_DENIED;
    }

    let l2eq = name_value.find('=').unwrap_or(name_value.len());
    if l2eq == 0 {
        pam_syslog!(&utpamh, LOG_ERR, "utpam_putenv: bad variable",);
        return PAM_BAD_ITEM;
    }
    match utpamh.env {
        Some(ref mut env) => {
            if env.list.is_empty() {
                pam_syslog!(&utpamh, LOG_ERR, "utpam_putenv: no env-list found",);
                return PAM_ABORT;
            }
            let item = utpam_search_env(env, &name_value[..l2eq], l2eq);

            if let Some('=') = name_value.chars().nth(l2eq) {
                if item == -1 {
                    //添加一个新的环境变量
                    env.requested += 1;
                    env.list.push(String::from(name_value));
                } else {
                    //更新现有的环境变量
                    env.list[item as usize] = String::from(name_value);
                }
            } else {
                // deleting
                if item == -1 {
                    pam_syslog!(
                        &utpamh,
                        LOG_ERR,
                        "utpam_putenv: delete non-existent entry; {}",
                        name_value
                    );
                    return PAM_BAD_ITEM;
                }

                env.list.remove(item as usize);
                env.requested -= 1;
            }
            PAM_SUCCESS
        }
        None => {
            pam_syslog!(&utpamh, LOG_ERR, "utpam_putenv: no env found",);
            PAM_ABORT
        }
    }
}

pub fn utpam_getenv(utpamh: &mut Option<Box<UtpamHandle>>, name: &str) -> Option<String> {
    let utpamh = match utpamh {
        Some(ref mut value) => value,
        None => return None,
    };
    if name.is_empty() {
        pam_syslog!(&utpamh, LOG_ERR, "utpam_getenv: no variable indicated",);
        return None;
    }
    let env = match &utpamh.env {
        Some(env) => env,
        None => {
            pam_syslog!(&utpamh, LOG_ERR, "utpam_getenv: no env found",);
            return None;
        }
    };
    if env.list.is_empty() {
        pam_syslog!(&utpamh, LOG_ERR, "utpam_getenv: no env-list found",);
        return None;
    }

    let item = utpam_search_env(env, name, name.len());
    if item != -1 {
        let env_str = &env.list[item as usize];
        let value_start = name.len() + 1; // 跳过名称部分和等号
        Some((env_str[value_start..]).to_string())
    } else {
        println!("env-item: {}, not found", name);
        None
    }
}

fn copy_env(utpamh: &mut Box<UtpamHandle>) -> Option<Vec<String>> {
    // 创建一个新的环境变量列表
    let mut dump;

    match utpamh.env {
        Some(ref env) => {
            let i = env.requested;
            let env_list = &env.list;
            dump = Vec::with_capacity(i);

            for j in (0..i).rev() {
                if let Some(env_str) = env_list.get(j) {
                    // 尝试复制环境变量
                    if let Some(dup_str) = utpam_strdup(env_str) {
                        dump.push(dup_str);
                    } else {
                        // 内存分配失败，返回 None
                        return None;
                    }
                }
            }
            // 因为是从后向前遍历的，所以需要反转 dump 以恢复原来的顺序
            dump.reverse();
        }
        None => return None,
    }
    // 释放旧的环境变量列表
    utpamh.env = None;

    Some(dump)
}

pub fn utpam_getenvlist(utpamh: &mut Option<Box<UtpamHandle>>) -> Option<Vec<String>> {
    //检查utpamh是否为空
    let utpamh = match utpamh {
        Some(ref mut value) => value,
        None => return None,
    };

    let env = match &utpamh.env {
        Some(env) => env,
        None => {
            pam_syslog!(&utpamh, LOG_ERR, "utpam_getenvlist: no env found",);
            return None;
        }
    };
    if env.list.is_empty() {
        pam_syslog!(&utpamh, LOG_ERR, "utpam_getenvlist: no env-list found",);
        return None;
    }

    if env.requested as i32 > env.entries {
        pam_syslog!(
            &utpamh,
            LOG_ERR,
            "utpam_getenvlist: environment corruptiond",
        );
        return None;
    }

    for i in (0..env.requested).rev() {
        if env.list[i].is_empty() {
            pam_syslog!(&utpamh, LOG_ERR, "utpam_getenvlist: environment broken",);
            return None;
        }
    }

    copy_env(utpamh)
}
