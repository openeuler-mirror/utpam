/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#![allow(non_camel_case_types)]

use std::env;
use std::ffi::CString;
use std::ptr::NonNull;
pub const _PAMC_DEFAULT_TOP_FD: i32 = 10;
pub const PAMC_SYSTEM_AGENT_PATH: &str = "/lib/pamc:/usr/lib/pamc";
pub const PAMC_SYSTEM_AGENT_SEPARATOR: char = ':';

#[derive(Clone)]
pub struct pamc_handle_t {
    pub current: Option<NonNull<PamcAgentS>>,
    pub chain: Option<NonNull<PamcAgentS>>,
    pub blocked_agents: Option<NonNull<PamcBlockedS>>,
    pub max_path: i32,
    pub agent_paths: Vec<String>,
    pub combined_status: i32,
    pub highest_fd_to_close: i32,
}

#[derive(Clone)]
pub struct PamcAgentS {
    pub id: CString,
    pub id_length: i32,
    pub next: Option<NonNull<PamcAgentS>>,
    pub writer: i32,
    pub reader: i32,
    pub pid: i32, // Assuming pid_t is i32
}

#[derive(Clone)]
pub struct PamcBlockedS {
    pub id: CString,
    pub next: Option<NonNull<PamcBlockedS>>,
}

//static，释放pch.agent_paths中内容，rust机制，函数可为空；
fn __pamc_delete_path_list(_pch: Option<pamc_handle_t>) {}

//函数大致功能，初始化pam控制框架，设置默认的代理路径，及使用的哪些代理路径；
//完成程度基本100%，有一处是否需要在转换后String类型的字符串后添加\0。
#[no_mangle]
pub fn utpamc_start() -> Option<pamc_handle_t> {
    let mut i: i32 = 0;
    let mut this: i32;
    let mut pch: Box<Option<pamc_handle_t>> = Box::new(None);

    //分配PAM结构体内存，由于Box，可省略；
    //pch = calloc(1,sizeof(struct pamc_handle_s));

    if pch.is_none() {
        //D(("no memory for *pch"));
        return None;
    }

    //解开关于文件描述符的限制；
    if let Some(ref mut handle) = *pch {
        handle.highest_fd_to_close = _PAMC_DEFAULT_TOP_FD;
    }

    //获取环境变量中的代理路径getenv agent_path；
    //c中getenv()由标准库env::var替代；
    //unwrap_or方法用于处理option、result类型，如some/OK，返回value，如Err/none，则返回unwrap_or提供的默认值；
    let default_p = env::var("PAMC_AGENT_PATH");
    let default_path = default_p.unwrap_or(PAMC_SYSTEM_AGENT_PATH.to_string());

    //计算代理路径的个数，以“:”分割, ex: /lib/:/usr/....,存于count；
    //chars().nth()方法用于在串中获取字符；
    let mut count = 1;
    while i < default_path.len() as i32 {
        if default_path.chars().nth(i as usize) == Some(PAMC_SYSTEM_AGENT_SEPARATOR) {
            count += 1;
        }
        i += 1;
    }

    //'drop_pch: loop {
    //由于结构体pch中成员agent_paths为Vec<String>类型，
    //所以不需要再为代理路径(agent_paths)进行内存分配等操作；
    //pch->agent_paths = calloc(count+1, sizeof(char *));

    //以下代码是判断是否为agent_paths分配成功，考虑堆上分配的Vec<String>,代码可以省略；
    /*match &*pch {
        Some(handle) => {
            if handle.agent_paths.is_empty() {
                //D(("no memory for path list"));
                return None;
            };
        }
        None => {}
    }*/

    //解析代理路径
    i = 0;
    let last = i;
    this = last;
    //this = last = i;
    while default_path.chars().nth(i as usize).is_some() || i != last {
        if default_path.chars().nth(i as usize) == Some(PAMC_SYSTEM_AGENT_SEPARATOR)
            || default_path.chars().nth(i as usize).is_none()
        {
            let length: i32 = 0;

            //pch->agent_paths[this] = malloc(length = 1+i-last);
            //为代理路径分配内存，Vec类型，无需进行内存申请操作；

            //Vec<String>类型的agent_paths,所以agent_paths[this]不为空，以下代码可以省略；
            /*match &*pch {
                Some(handle) => {
                    if handle.agent_paths[this as usize].is_empty() {
                        //D(no memory for next path);
                        break 'drop_pch;
                    }


                    /*if handle.agent_paths.chars().nth(this as usize).is_none() {
                        //D(no memory for next path);
                        break 'drop_pch;
                    }*/
                }
                None => {}
            }*/

            //memcpy(pch->agent_paths[this], default_path + last, i-last);
            //实现解构pch，将default_path的last到i-last拷贝至pch.agent_paths[this]中；
            if let Some(handle) = pch.as_mut() {
                let start = last as usize;
                let end = (i - last) as usize;

                if start + end <= default_path.len() {
                    let slice = &default_path[start..start + end];
                    handle.agent_paths[this as usize] = slice.to_string();
                }
            }
            //pch->agent_paths[this][i-last] = '\0';
            //原意是在拷贝到目的字符串后添加结束符，

            if let Some(ref mut handle) = *pch {
                if length > handle.max_path {
                    handle.max_path = length;
                }
            }

            if {
                this += 1;
                this
            } == count
            {
                //处理完所有路径退出
                break;
            }
            let _last = i + 1;
        } else {
            i += 1;
        }
    } //while
    *pch

    //drop_list: drop_pch:
    //c中由于申请内存失败会跳转到此，来返回空，
    //rust使用动态变量存储，不需要申请内存操作，所以不会跳转到此分支，
    //如此，此函数默认返回pam结构体pch。

    //free(pch);
    //None
}

pub const PAM_BPC_FALSE: u8 = 0;
//释放pamc_handle_t类型指针，并关闭资源，
//失败返回false宏，成功返回retval(经pamc_shutdown_agents函数)
#[no_mangle]
pub fn pamc_end(mut pch_p: Option<&mut pamc_handle_t>) -> u32 {
    let retval: u32 = 0;

    if pch_p.is_none() {
        //D(called with no pch_p);
        return PAM_BPC_FALSE as u32;
    }

    if let Some(ref mut _pch_ref) = pch_p {
    } else {
        //D(called with no *pch_p);
        return PAM_BPC_FALSE as u32;
    }

    //D("removing path_list");
    //__pamc_delete_path_list(*pch_p); 当前文件实现，功能是释放内存，可忽略此行；

    //D(shutting down agents);
    //retval = __pamc_shutdown_agents(*pch_p); //此函数为关闭代理程序，还未实现。

    //D("freeing *pch_p");
    //free(*pch_p);
    //*pch_p = null;

    retval
}
