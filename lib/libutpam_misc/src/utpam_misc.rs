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
    clippy::never_loop
)]

use std::any::Any;
use std::io;
use std::io::Write;
use utpam::common::{UtpamMessage, UtpamResponse};

use utpam::common::PAM_CONV_ERR;
use utpam::common::PAM_SUCCESS;

pub const PAM_PROMPT_ECHO_ON: isize = 2;
pub const PAM_PROMPT_ECHO_OFF: isize = 1;
pub const PAM_BINARY_PROMPT: isize = 7;
pub const PAM_ERROR_MSG: isize = 3;
pub const PAM_TEXT_INFO: isize = 4;

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
    num_msg: isize,
    msgm: &[UtpamMessage],
    response: &mut Option<Vec<UtpamResponse>>,
    appdata_ptr: Option<Box<dyn Any>>,
) -> isize {
    let mut count = 0_usize;

    if num_msg <= 0 {
        return PAM_CONV_ERR as isize;
    }

    println!("allocating empty response structure array."); //debug

    let mut reply: Vec<UtpamResponse> = Vec::with_capacity(num_msg as usize);
    for _ in 0..num_msg {
        reply.push(UtpamResponse {
            resp: Vec::new(),
            resp_retcode: 0,
        });
    }

    println!("entering conversation function."); //debug

    'failed_conversation: loop {
        loop {
            if count >= num_msg as usize {
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
        return PAM_SUCCESS as isize;
    } //end of goto,failed_conversation

    println!("the conversation failed"); //debug

    if !reply.is_empty() {
        count = 0;
        loop {
            if count >= num_msg as usize {
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
    PAM_CONV_ERR as isize
}
