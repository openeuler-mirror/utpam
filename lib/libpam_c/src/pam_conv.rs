/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::unwrap_or_default, clippy::not_unsafe_ptr_arg_deref)]
use libc::*;
use std::any::Any;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use std::slice;
use utpam::common::*;

type Misc = unsafe extern "C" fn(
    libc::c_int,
    *mut *const PamMessage,
    *mut *mut PamResponse,
    *mut libc::c_void,
) -> libc::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PamConv {
    pub conv: Option<Misc>,
    pub appdata_ptr: *mut libc::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PamResponse {
    pub resp: *mut libc::c_char,
    pub resp_retcode: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PamMessage {
    pub msg_style: libc::c_int,
    pub msg: *const libc::c_char,
}

// 将C语言的PamConv结构体转换为Rust的UtpamConv结构体
pub fn pamconv_to_utpamconv(pamconv: *const PamConv) -> UtpamConv {
    /*创建闭包并将它包装在一个 Box 中
    闭包的签名与 MiscConv 类型一致，接受消息数量、消息列表、响应列表的可变引用和应用数据，并返回一个 u8 值
    */
    let conv = Box::new(
        move |num_msg: usize,
              msg: &[UtpamMessage],
              resp: &mut Option<Vec<UtpamResponse>>,
              appdata_ptr: Option<Rc<dyn Any>>|
              -> u8 {
            //将 Rust 的 Option<Rc<dyn Any>> 类型的应用数据转换为 C 语言的 *mut c_void 指针
            let c_appdata_ptr = if let Some(appdata) = appdata_ptr {
                //使用 Rc::into_raw 将 Rc 转换为原始指针
                Rc::into_raw(appdata) as *mut c_void
            } else {
                //返回一个空指针
                ptr::null_mut()
            };

            //将 Rust 的 UtpamMessage 列表转换为 C 语言的 PamMessage 列表
            let mut pam_messages: Vec<*const PamMessage> = msg
                .iter()
                .map(|m| {
                    let c_msg = CString::new(m.msg.clone()).unwrap().into_raw();
                    let pam_message = PamMessage {
                        msg_style: m.msg_style as c_int,
                        msg: c_msg,
                    };
                    //包装在Box 中，然后转换为原始指针
                    Box::into_raw(Box::new(pam_message)) as *const PamMessage
                })
                .collect(); //将所有的 PamMessage 原始指针收集到一个向量中

            //表示初始状态下没有响应
            let pam_responses: *mut *mut PamResponse = ptr::null_mut();
            //调用 C 语言的回调函数 pamconv.conv，传入消息数量、消息列表指针、响应列表指针和应用数据指针。
            let result = unsafe {
                ((*pamconv).conv.unwrap())(
                    num_msg as c_int,
                    pam_messages.as_mut_ptr(),
                    pam_responses,
                    c_appdata_ptr,
                )
            };

            //如果 pam_responses 不为空，表示有响应生成
            if !pam_responses.is_null() {
                let c_responses = unsafe { slice::from_raw_parts(*pam_responses, num_msg) };
                let mut rust_responses = Vec::with_capacity(num_msg);

                for c_response in c_responses {
                    let c_str = unsafe { CString::from_raw((c_response).resp) };
                    let resp_str = c_str.to_string_lossy().into_owned();
                    rust_responses.push(UtpamResponse {
                        resp: vec![resp_str],
                        resp_retcode: (c_response).resp_retcode as isize,
                    });
                }

                *resp = Some(rust_responses);
            }

            //如果 c_appdata_ptr 不为空，表示有应用数据需要释放
            if !c_appdata_ptr.is_null() {
                unsafe {
                    let _ = Rc::from_raw(c_appdata_ptr as *const dyn Any);
                }; // 手动释放内存
            }

            result as u8
        },
    );

    let appdata_any = if unsafe { (*pamconv).appdata_ptr.is_null() } {
        None
    } else {
        Some(unsafe { Rc::from_raw((*pamconv).appdata_ptr as *const dyn Any) })
    };

    UtpamConv {
        conv: Some(conv),
        appdata_ptr: appdata_any,
    }
}
