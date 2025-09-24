/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(
    dead_code,
    unused_mut,
    non_camel_case_types,
    unused_variables,
    unused_assignments,
    unused_comparisons,
    non_upper_case_globals,
    clippy::absurd_extreme_comparisons,
    clippy::never_loop,
    unused_imports,
    unused_must_use,
    clippy::write_with_newline,
    clippy::redundant_static_lifetimes
)]

use std::any::Any;
use std::io;
use std::io::Write;
use std::rc::Rc;
use utpam::common::*;
use utpam::common::{UtpamMessage, UtpamResponse};

//fn set_alarm
use crate::utpam_misc::signal::SigSet;
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, Signal};
use nix::unistd::alarm;
use std::time::Duration;

#[no_mangle]
extern "C" fn time_is_up(_: i32) {
    unsafe {
        expired = 1;
    }
}

//用于(模拟)还原信号程序的注册函数，空函数；
#[no_mangle]
extern "C" fn doing_nothing(_: i32) {}

//函数大致功能：设置定时器，时间到达时出发相应信号处理函数；
//函数签名处少了一个参数(原始信号处理结构的指针)，由于rust里面的signal::sigaction后不支持第三个参数，
//所以无法传入原始指针结构体,此函数也仅被当前文件的read_string函数调用；
//目前实现方案为在以下函数中模拟一个(原始)信号处理程序(c端的struct sigaction也为初始状态)；
//参数delay为延迟时间(s),用于设置定时器的定时时间；返回值1均为失败(信号及定时器失败)；
#[no_mangle]
pub fn set_alarm(delay: i32) -> i32 {
    // 创建一个新的信号处理程序
    let new_action = SigAction::new(
        SigHandler::Handler(time_is_up),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    //创建一个模拟(旧的)信号处理程序，用于恢复至初始化信号状态，
    let virt_old_action = SigAction::new(
        SigHandler::Handler(doing_nothing),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    // 尝试设置新的信号处理方式，接收到SIGALRM(定时器到时)信号，即执行new_action中内容，
    // 主要看Handler后的信号注册函数(time_is_up);
    unsafe {
        match signal::sigaction(signal::Signal::SIGALRM, &new_action) {
            Ok(_) => (),
            Err(_) => return 1, // 设置信号失败
        }
    }

    // 尝试设置定时器,alarm::set()后的None表示没有设定定时器，Some(secs) secs>0表示定时器剩余时间；
    // 以下match表示如果返回正数，则说明定时器已经设置，那么将信号程序还原，且返回1；
    match alarm::set(delay as u32) {
        None => (),
        Some(secs) => {
            if secs > 0 {
                unsafe {
                    signal::sigaction(signal::Signal::SIGALRM, &virt_old_action);
                }
                return 1;
            }
        }
    }
    0 // 所有操作成功
}
//signal

struct MyStruct {
    length: u32,
    control: u8,
}
type pamc_bp_t = Option<Box<MyStruct>>;
//type pamc_bp_t = Vec<MyStruct>;
type pam_binary_handler_fn_t = fn(appdata: Option<Box<dyn Any>>, prompt_p: *mut pamc_bp_t) -> i32;
static pam_binary_handler_fn: Option<pam_binary_handler_fn_t> = None;

//函数大致功能：处理pam消息的交互，主要检查消息数量(num_msg)是否有效，遍历消息并处理。
//目前未完成两块内容，一是调用函数read_string,二是"///////"部分，两个宏展开的内容没完成。

//函数参数和返回值不变即可
#[no_mangle]
pub fn misc_conv(
    num_msg: usize,
    msgm: &[UtpamMessage],
    response: &mut Option<Vec<UtpamResponse>>,
    appdata_ptr: Option<Rc<dyn Any>>,
) -> u8 {
    let mut count = 0_usize;

    if num_msg <= 0 {
        return PAM_CONV_ERR;
    }

    println!("allocating empty response structure array."); //debug

    let mut reply: Vec<UtpamResponse> = Vec::with_capacity(num_msg);
    for _ in 0..num_msg {
        reply.push(UtpamResponse {
            resp: Vec::new(),
            resp_retcode: 0,
        });
    }

    println!("entering conversation function."); //debug

    'failed_conversation: loop {
        loop {
            if count >= num_msg {
                break;
            }
            //let mut string: Option<String> = None; remove
            let mut string: Vec<String> = Vec::new();
            let mut nc: u64 = 0;

            match msgm[count].msg_style {
                PAM_PROMPT_ECHO_OFF => {
                    //nc = read_string(CONV_ECHO_OFF,msgm[count].msg,&string);
                    if nc < 0 {
                        break 'failed_conversation;
                    }
                }
                PAM_PROMPT_ECHO_ON => {
                    //nc = read_string(CONV_ECHO_ON,msgm[count].msg,&string);
                    if nc < 0 {
                        break 'failed_conversation;
                    }
                }
                PAM_ERROR_MSG => {
                    //let res = write!(io::stderr(),"{:?}\n",msgm[count].msg);
                    let res = writeln!(io::stderr(), "{:?}", msgm[count].msg);
                    if res.is_err() {
                        break 'failed_conversation;
                    }
                }
                PAM_TEXT_INFO => {
                    //let res = write!(io::stdout(),"{:?}\n",msgm[count].msg);
                    let res = writeln!(io::stdout(), "{:?}", msgm[count].msg);
                    if res.is_err() {
                        break 'failed_conversation;
                    }
                }
                PAM_BINARY_PROMPT => {
                    let mut binary_prompt: pamc_bp_t = None;
                    if msgm[count].msg.is_empty() || pam_binary_handler_fn.is_none() {
                        break 'failed_conversation;
                    }
                    /////////////
                    //if let Some(handler) = pam_binary_handler_fn {
                    //    if handler(appdata_ptr,&mut binary_prompt) != PAM_SUCCESS || binary_prompt.is_none() {
                    //       break 'failed_conversation;
                    //    }
                    //}
                    if let Some(boxed_mystruct) = binary_prompt {
                        let mystruct = *boxed_mystruct;
                        string.push(mystruct.length.to_string());
                        string.push(mystruct.control.to_string());
                    } else {
                        string.clear();
                    }
                    //string = binary_prompt;
                    binary_prompt = None;
                }
                _ => {
                    //let res = write!(io::stderr(),"erroneous conversation ({})\n",msgm[count].msg_style);
                    let res = writeln!(
                        io::stderr(),
                        "erroneous conversation ({})",
                        msgm[count].msg_style
                    );
                    break 'failed_conversation;
                }
            } //match

            if !string.is_empty() {
                reply[count].resp_retcode = 0;
                //reply[count].resp = string; remove
                reply[count].resp = string.clone();
                //if let Some(s) = string {   3 remove
                //    reply[count].resp.push(s);
                //}
                string.clear();
            }
            count += 1;
        } //loop

        *response = Some(reply);
        //reply = null;//离开作用域自动释放；
        return PAM_SUCCESS;
    } //end of goto,failed_conversation

    println!("the conversation failed"); //debug

    if !reply.is_empty() {
        count = 0;
        loop {
            if count >= num_msg {
                break;
            }
            if reply[count].resp.is_empty() {
                count += 1;
                continue;
            }

            match msgm[count].msg_style {
                PAM_PROMPT_ECHO_ON | PAM_PROMPT_ECHO_OFF => {
                    //pam_overwrite_string(reply[count].resp)；free(); 清理资源，rust机制自动回收
                }
                PAM_BINARY_PROMPT => {
                    //let bt_ptr : Option<Box<dyn Any>> = None; 2 remove
                    //bt_ptr = reply[count].resp;
                    let mut bt_ptr: Option<Vec<String>> = None;
                    bt_ptr = Some(reply[count].resp.clone());
                    //pam_binary_handler_free(appdata_ptr,but_)ptr)
                }
                PAM_ERROR_MSG | PAM_TEXT_INFO => {
                    //reply[count].resp.clear(); rust会自动回收内存
                }
                _ => {}
            }

            reply[count].resp.clear();
            count += 1;
        }
    }
    PAM_CONV_ERR
}

//utget_delay函数被用于read_string函数，主要作用为获取时间差值，来计算延迟
//主要涉及两个时间概念：警告时间，终止时间
//首先会获取当前时间，分别比较当前时间与上述两种时间，返回的是当前时间与警告（终止）时间的差值
//从整个项目来看，应该是跟安全性有关，在read_string读取用户输入时进行一定的延迟，类似防暴力破解的功能
use std::time::SystemTime;

use std::ffi::CString;
use std::time::UNIX_EPOCH;

const pam_misc_conv_warn_line: &'static str = "...Time is running out...\n";
const pam_misc_conv_die_line: &'static str = "...Sorry, your time is up!\n";
static mut expired: i32 = 0;
#[no_mangle]
pub fn utget_delay() -> i32 {
    let mut pam_misc_conv_warn_time: u64 = 0; //警告时间，到达时间，如打印警告信息；
    let mut pam_misc_conv_die_time: u64 = 0; //终止时间，如退出或关闭某个服务；
    let mut pam_misc_conv_died: i64 = 0;
    let mut now = 0_isize;

    unsafe {
        expired = 0;
    }
    // 获取当前系统时间
    let now_tt = SystemTime::now();
    // 计算自UNIX_EPOCH以来的持续时间
    let since_epoch = now_tt
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    // 将持续时间转换为秒数
    let now = since_epoch.as_secs();

    if pam_misc_conv_die_time != 0 && now >= pam_misc_conv_die_time {
        //write!(io::stderr(),"{:?}\n",pam_misc_conv_die_line);
        writeln!(io::stderr(), "{:?}", pam_misc_conv_die_line);
        pam_misc_conv_died = 1;
        return -1;
    }

    if pam_misc_conv_warn_time != 0 && now >= pam_misc_conv_warn_time {
        //write!(io::stderr(),"{:?}\n",pam_misc_conv_warn_line);
        writeln!(io::stderr(), "{:?}", pam_misc_conv_warn_line);
        pam_misc_conv_warn_time = 0;
        if pam_misc_conv_die_time != 0 {
            return (pam_misc_conv_die_time - now) as i32;
        } else {
            return 0;
        }
    }

    if pam_misc_conv_warn_time != 0 {
        (pam_misc_conv_warn_time - now) as i32
    } else if pam_misc_conv_die_time != 0 {
        (pam_misc_conv_die_time - now) as i32
    } else {
        return 0;
    }
}
