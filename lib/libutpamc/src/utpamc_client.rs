/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(
    unused_mut,
    unused_imports,
    unused_variables,
    unreachable_code,
    unused_assignments,
    clippy::unused_unit,
    non_camel_case_types,
    clippy::needless_return,
    clippy::nonminimal_bool,
    clippy::needless_return
)]

use crate::WEXITSTATUS;
use crate::WIFEXITED;
use nix::fcntl;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::close;
use nix::unistd::Pid;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::ptr;
use std::ptr::NonNull;
use utpam::D;

pub const PAM_BPC_TRUE: u32 = 1;
pub const PAM_BPC_FALSE: u32 = 0;
pub const _PAMC_DEFAULT_TOP_FD: i32 = 10;
pub const PAMC_SYSTEM_AGENT_SEPARATOR: char = ':';
pub const PAMC_SYSTEM_AGENT_PATH: &str = "/lib/pamc:/usr/lib/pamc";

#[derive(Clone)]
pub struct pamc_handle_t {
    pub current: Option<Box<PamcAgentS>>,
    pub chain: Option<Box<PamcAgentS>>,
    pub blocked_agents: Option<Box<PamcBlockedS>>,
    pub max_path: i32,
    pub agent_paths: Vec<String>,
    pub combined_status: i32,
    pub highest_fd_to_close: i32,
}

#[derive(Clone)]
pub struct pamc_agent_s {
    pub id: Vec<String>,
    pub id_length: usize,
    pub next: Option<Box<pamc_agent_s>>,
    pub writer: std::os::unix::io::RawFd, // 文件描述符
    pub reader: std::os::unix::io::RawFd,
    pub pid: u32, // 进程 ID
}

#[derive(Clone)]
pub struct PamcAgentS {
    pub id: CString,
    pub id_length: i32,
    pub next: Option<Box<PamcAgentS>>,
    pub writer: i32,
    pub reader: i32,
    pub pid: i32, // Assuming pid_t is i32
}

#[derive(Clone)]
pub struct PamcBlockedS {
    pub id: CString,
    pub next: Option<NonNull<PamcBlockedS>>,
}

//static，释放pch.agent_paths中内容，rust机制，所以此处设计为空函数；
fn __pamc_delete_path_list(pch: Option<pamc_handle_t>) -> () {}

//函数大致功能，初始化pam控制框架，设置默认的代理路径，及使用的哪些代理路径；
#[no_mangle]
pub fn utpamc_start() -> Option<pamc_handle_t> {
    let mut i: i32 = 0;
    let mut count: i32 = 0;
    let mut last: i32 = 0;
    let mut this: i32 = 0;
    let default_path: Option<&'static str> = None;
    let mut pch: Box<Option<pamc_handle_t>> = Box::new(None);

    if pch.is_none() {
        D!("no memory for *pch");
        return None;
    }

    //解开关于文件描述符的限制；
    if let Some(ref mut handle) = *pch {
        handle.highest_fd_to_close = _PAMC_DEFAULT_TOP_FD;
    }

    //获取环境变量中的代理路径getenv agent_path；
    let default_p = env::var("PAMC_AGENT_PATH");
    let default_path = default_p.unwrap_or(PAMC_SYSTEM_AGENT_PATH.to_string());

    //计算代理路径的个数，以“:”分割, ex: /lib/:/usr/....,存于count；
    let mut count = 1;
    while i < default_path.len() as i32 {
        if default_path.chars().nth(i as usize) == Some(PAMC_SYSTEM_AGENT_SEPARATOR) {
            count += 1;
        }
        i += 1;
    }

    //解析代理路径
    i = 0;
    last = i;
    this = last;
    while default_path.chars().nth(i as usize).is_some() || i != last {
        if default_path.chars().nth(i as usize) == Some(PAMC_SYSTEM_AGENT_SEPARATOR)
            || !default_path.chars().nth(i as usize).is_some()
        {
            let mut length: i32 = 0;

            //memcpy(pch->agent_paths[this], default_path + last, i-last);
            //解构pch，将default_path的last到i-last拷贝至pch.agent_paths[this]中；
            if let Some(handle) = pch.as_mut() {
                let start = last as usize;
                let end = (i - last) as usize;

                if start + end <= default_path.len() {
                    let slice = &default_path[start..start + end];
                    handle.agent_paths[this as usize] = slice.to_string();
                }
            }

            if let Some(ref mut handle) = *pch {
                if length > handle.max_path {
                    handle.max_path = length;
                }
            }

            if ({
                this += 1;
                this
            } == count)
            {
                break;
            }

            let last = i + 1;
        } else {
            i += 1;
        }
    }
    return *pch;
}

#[no_mangle]
pub fn utpamc_end(mut pch_p: Option<&mut pamc_handle_t>) -> u32 {
    let mut retval: u32 = 0;

    if pch_p.is_none() {
        D!("called with no pch_p");
        return PAM_BPC_FALSE;
    }

    if let Some(ref mut pch_ref) = pch_p {
    } else {
        D!("called with no *pch_p");
        return PAM_BPC_FALSE;
    }

    D!("removing path_list");
    D!("shutting down agents");
    retval = __pamc_shutdown_agents(pch_p.cloned()); //关闭代理程序;

    D!("freeing *pch_p");

    return retval;
}

#[no_mangle]
fn update_chain(pch: Option<pamc_handle_t>) {
    if let Some(mut handle) = pch {
        if let Some(ref mut current_chain) = handle.chain {
            if let Some(next) = current_chain.next.take() {
                handle.chain = current_chain.next.take();
            }
        }
    }
}

#[no_mangle]
pub fn __pamc_shutdown_agents(mut pch: Option<pamc_handle_t>) -> u32 {
    let mut retval = PAM_BPC_TRUE;
    D!("called");

    while let Some(ref pchh) = pch {
        if let Some(chainn) = pch.clone().unwrap().chain {
            let mut pid: u32 = 0;
            let mut status: u32 = 0;
            let mut this: Option<Box<PamcAgentS>> = None;

            this = Some(chainn);
            D!("cleaning up agent: {}", this);

            update_chain(Some(pchh.clone())); //pch->chain = pch->chain->next下一跳
            this.clone().unwrap().next = None;
            D!("cleaning up agent: {}", this.id);

            close(this.clone().unwrap().writer).expect("Failed to close file descriptor");
            this.clone().unwrap().writer = -1;
            close(this.clone().unwrap().reader).expect("Failed to close file descriptor");
            this.clone().unwrap().reader = -1;

            if let Some(ref agent) = this {
                let pid = Pid::from_raw(agent.pid);
                match waitpid(pid, None) {
                    Ok(WaitStatus::Exited(pidd, exit_status)) => {
                        if pidd.as_raw() == agent.pid {
                            D!(
                                "is exit: {},exit val: {}",
                                WIFEXITED!(status),
                                WEXITSTATUS!(status)
                            );
                            if !(WIFEXITED!(status) == 0 && WEXITSTATUS!(status) == 0) {
                                retval = PAM_BPC_FALSE;
                            }
                        } else {
                            D!(
                                "problem shutting down agent ({}): pid({}) != waitpid({})!?",
                                this.id,
                                this.pid,
                                pid
                            );
                            retval = PAM_BPC_FALSE;
                        }
                    }

                    Ok(WaitStatus::Signaled(pidd, signal, core_dumped)) => {
                        println!("Child with PID {} was signaled with {}.", pidd, signal);
                    }
                    Ok(WaitStatus::Stopped(pidd, signal)) => {
                        println!("Child with PID {} was stopped by signal {}", pidd, signal);
                    }
                    Err(e) => println!("Error waiting for child: {}", e),
                    _ => todo!(),
                }
            }

            if let Some(ref mut agent) = this {
                agent.pid = 0
            }

            if let Some(ref mut agentt) = this {
                pid = agentt.pid as u32
            }
        }
    }

    return retval;
}
