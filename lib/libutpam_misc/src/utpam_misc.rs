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
    clippy::redundant_static_lifetimes,
    clippy::manual_unwrap_or_default,
    clippy::single_match,
    clippy::ptr_arg,
    clippy::unused_unit
)]

use std::any::Any;
use std::io;
use std::io::Write;
use std::rc::Rc;
use utpam::common::*;
use utpam::common::{UtpamMessage, UtpamResponse};
use utpam::utpam_overwrite_string;
use utpamc::*;
//fn read_string
use nix::sys::termios::LocalFlags;
use nix::sys::termios::Termios;
//use nix::sys::termios::{self, Termios, ECHO};
use nix::unistd::isatty;
use std::io::Read;
use std::os::unix::io::RawFd;
use zeroize::Zeroize;
pub const CONV_ECHO_OFF: u8 = 0;
pub const CONV_ECHO_ON: u8 = 1;
//use nix::sys::termios::NCCS;
use nix::sys::termios::SetArg::TCSAFLUSH;
pub const NCCS: usize = 32;
pub const PAM_MISC_CONV_BUFSIZE: usize = 4096;
pub const STDIN_FILENO: u8 = 0;
pub const ECHO: u8 = 10;
pub const INPUTSIZE: i32 = 4096;
use std::sync::MutexGuard;

use nix::errno::Errno;
use nix::sys::termios;

pub struct __sigset_t {
    pub val: [u8; 16],
}

#[no_mangle]
fn read_string(echo: u8, prompt: String, retstr: &mut Vec<String>) -> i8 {
    let mut fd = RawFd::from(0);
    let mut fdd = RawFd::from(0);
    let mut term_before = match termios::tcgetattr(fd) {
        Ok(termios) => termios,
        Err(_) => return -1,
    };
    let mut term_tmp = match termios::tcgetattr(fdd) {
        Ok(termios) => termios,
        Err(_) => return -1,
    };

    let mut line: [u8; PAM_MISC_CONV_BUFSIZE] = [0; PAM_MISC_CONV_BUFSIZE];
    //struct sigaction old_sig;//c中sigaction目前用nix实现，相关结构体定义，暂时不定义相关变量；
    let mut delay: i32 = 0;
    let mut nc: i32 = -1;
    let mut have_term: i8 = 0;

    //D(("called with echo='%s', prompt='%s'.", echo ? "ON":"OFF" , prompt));

    let is_term = isatty(RawFd::from(0)).unwrap_or(false);
    if is_term {
        match termios::tcgetattr(STDIN_FILENO as i32) {
            Ok(_) => {
                // 如果成功，继续执行其他操作
                let mut term_tmp = term_before.clone(); //memcpy...
                                                        //let echo_bool_f = echo != 0;
                if echo == 0 {
                    let echo_mask = LocalFlags::from_bits(ECHO as u32).unwrap();
                    term_tmp.local_flags &= !(echo_mask);
                    //term_tmp.local_flags &=!ECHO;
                    //由于ECHO使用的termios中变量，类型不一致，所以需要使用上面方法(localflags::from_bits)转一下类型；
                }
                have_term = 1;

                // 初始化信号集 nset
                let mut nset = signal::SigSet::empty();

                // 将 SIGTSTP 添加到 nset
                //signal::sigaddset(&mut nset, signal::SIGTSTP);
                nset.add(signal::SIGTSTP);

                // 阻塞 nset 中的信号，并将当前的信号掩码保存到 oset 中
                let mut oset = signal::SigSet::empty();
            }
            Err(_) => {
                //D(("<error: failed to get terminal settings>"));
                //*retstr = None; 程序结束后，指针会自动释放
                return -1;
            }
        }
    } else {
        //let echo_bool = echo != 0;
        if echo == 0 {
            //D(("<warning: cannot turn echo off>"));
        }
    }

    delay = utget_delay();

    'cleanexit: loop {
        while delay >= 0 {
            if have_term != 0 {
                termios::tcsetattr(STDIN_FILENO.into(), TCSAFLUSH, &term_tmp);
            }
            eprintln!("{}", prompt);

            if delay > 0 && (set_alarm(delay) != 0) {
                //D(("<failed to set alarm>"))
                break;
            } else {
                if have_term != 0 {
                    let stdin = io::stdin();
                    //nc = stdin.lock().read(&mut line[0..4095]);
                    match stdin.lock().read(&mut line[0..4095]) {
                        Ok(bytes_read) => {
                            nc = bytes_read as i32;
                        }
                        Err(_) => {
                            println!("Failed to read from stdin");
                        }
                    }
                } else {
                    nc = 0;
                    while nc < ((INPUTSIZE - 1) as isize).try_into().unwrap()
                        && line[nc as usize] != b'\n'
                    {
                        let mut rv: i32 = 0;
                        let stdin_1 = io::stdin();
                        match stdin_1.lock().read(&mut line[0..1]) {
                            Ok(bytes_read) => {
                                rv = bytes_read as i32;
                                if rv != 1 {
                                    if rv < 0 {
                                        //pam_overwrite_n(line,(unsigned int) nc)............
                                        utpam_overwrite_string!(line);
                                        nc = rv;
                                    }
                                    break;
                                }
                            }
                            Err(_) => {
                                println!("Failed to read from stdin");
                            }
                        }
                        nc += 1;
                    }
                } //line264 break

                if have_term != 0 {
                    termios::tcsetattr(STDIN_FILENO.into(), TCSAFLUSH, &term_tmp);
                    //let echo_1 = echo != 0;
                    unsafe {
                        if echo == 0 || expired != 0 {
                            eprintln!();
                        }
                    } //unsafe
                }

                if delay > 0 {
                    reset_alarm();
                }

                if unsafe { expired != 0 } {
                    delay = utget_delay();
                } else if nc > 0 {
                    //D(("we got some user input"));
                    if line[nc as usize - 1] == b'\n' {
                        nc -= 1;
                        line[nc as usize] = b'\0';
                    } else {
                        if echo != 0 {
                            eprintln!();
                        }
                        line[nc as usize] = b'\0';
                    }
                    for s in &line {
                        //strdup;复制字符串至retstr；
                        retstr.push(s.to_string());
                    }
                    //pam_overwrite_array(line);
                    utpam_overwrite_string!(line);
                    if (*retstr).is_empty() {
                        //D(("no memory for response string"));
                        nc -= 1;
                    }

                    break 'cleanexit;
                } else if nc == 0 {
                    //295
                    //D(("user did not want to type anything"))
                    //(*retstr) = NULL ;生命周期
                    if echo != 0 {
                        eprintln!();
                    }
                    break 'cleanexit;
                } else if nc == -1 {
                    //303
                    //D(("user did not want to type anything"))
                    if echo != 0 {
                        eprintln!();
                    }
                    //(*retstr) = NULL ;
                    break 'cleanexit;
                }
            }
        } //end of while

        //D(("the timer appears to have expired"))
        //*retstr = NULL; 退出程序会自动释放内存

        //pam_overwrite_array(line);
        utpam_overwrite_string!(line);

        break 'cleanexit;
    } //end of loop cleanexit;

    //cleanexit
    if have_term != 0 {
        //signal::sigprocmask(signal::SigmaskHow::SIG_SETMASK,&oset,None);
        termios::tcsetattr(STDIN_FILENO.into(), TCSAFLUSH, &term_before);
    }
    //return nc as i8;
    nc as i8 //cargo-clippy
}

//delete bin
//函数是字节序的转换作用
fn swap_bytes(mut num: u32) -> u32 {
    num = ((num & 0xFF000000) >> 24)
        | ((num & 0x00FF0000) >> 8)
        | ((num & 0x0000FF00) << 8)
        | ((num & 0x000000FF) << 24);
    num
}

//此函数主要是作为函数指针所指的函数，原c的第二个参数为裸指针，此处为引用，
//因为传入裸指针会导致函数中判断是否空及解构时存在问题，功能是释放二进制数据；
//原c仅仅一行PAM_BP_RENEW();
//-------------------宏展开，可能需要再仔细确认一下重构后内容；
#[no_mangle]
fn pam_misc_conv_delete_binary(
    appdata: Option<Box<dyn Any>>,
    mut delete_me: Option<Box<MyStruct>>,
) -> () {
    //PAM_BP_RENEW(delete_me,0,0);
    if let Some(ref mut inner) = delete_me {
        let swapped = swap_bytes(inner.length);
        //memset(delete_me,0,swapped)
        //返回的字节swapped作为memset第三个参数，但是memset在rust中可以利用rust覆盖；
        //PAM_BP_FREE(delete_me); rust中不需要释放资源
    } else {
        //delete_me.is_none()

        //PAM_BP_ASSERT("programming error, invalid binary prompt pointer");
        //此宏较简单，仅组包即可；
        let error_message = b"programming error, invalid binary prompt pointer\0";
        println!("{}({}): {:?}", file!(), line!(), error_message);
        std::process::exit(1);
    }
}

//end of delete bin

//函数原功能：取消报警且重置信号处理为默认行为；
//查阅相关资料，关于设置原始信号处理程序可以仍用SigAction::new()方法，
//只是参数修改为SigDfl即可；
//原c代码传入原始信号处理结构体，有个rust的库nix后，可以创建一个默认信号处理程序以代替原始信号处理程序，
//所以此处可以不用设置函数参数，此函数也仅用于当前文件的其他函数(read_string)
#[no_mangle]
fn reset_alarm() {
    alarm::set(0);

    //创建信号处理程序，重点为第一个参数，设置为default;
    let ori_action = SigAction::new(SigHandler::SigDfl, SaFlags::SA_RESTART, SigSet::empty());

    unsafe {
        match signal::sigaction(signal::Signal::SIGALRM, &ori_action) {
            Ok(_) => (),
            Err(_) => (), // 设置信号失败
        }
    }
}

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
    //创建一个模拟(旧的)信号处理程序，用于恢复至初始化信号状态，原来手动设计了个空函数以满足原始信号处理程序，
    //后发现第一个参数改为SigDfl就是将某信号的处理重置为默认行为；
    let virt_old_action = SigAction::new(SigHandler::SigDfl, SaFlags::SA_RESTART, SigSet::empty());

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
type pam_binary_handler_fn_t = fn(appdata: Option<Rc<dyn Any>>, prompt_p: &mut pamc_bp_t) -> i32;
static pam_binary_handler_fn: Option<pam_binary_handler_fn_t> = None;

type pam_binary_handler_free_t =
    fn(appdata: Option<Box<dyn Any>>, prompt_p: Option<Box<MyStruct>>) -> ();
static pam_binary_handler_free: Option<pam_binary_handler_free_t> =
    Some(pam_misc_conv_delete_binary);

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

    //D(allocating empty response structure array)

    let mut reply: Vec<UtpamResponse> = Vec::with_capacity(num_msg);
    for _ in 0..num_msg {
        //reply=calloc(num_msg)
        reply.push(UtpamResponse {
            resp: Vec::new(),
            resp_retcode: 0,
        });
    }
    if reply.is_empty() {
        //D(no memory for responses)
        return PAM_CONV_ERR;
    }

    //D(entering conversation function)

    'failed_conversation: loop {
        loop {
            if count >= num_msg {
                break;
            }
            //let mut string: Option<String> = None; remove
            let mut string: Vec<String> = Vec::new();
            let mut nc: i8 = 0;

            match msgm[count].msg_style {
                PAM_PROMPT_ECHO_OFF => {
                    nc = read_string(CONV_ECHO_OFF, msgm[count].msg.clone(), &mut string);
                    if nc < 0 {
                        break 'failed_conversation;
                    }
                }
                PAM_PROMPT_ECHO_ON => {
                    nc = read_string(CONV_ECHO_ON, msgm[count].msg.clone(), &mut string);
                    if nc < 0 {
                        break 'failed_conversation;
                    }
                }
                PAM_ERROR_MSG => {
                    let res = writeln!(io::stderr(), "{:?}", msgm[count].msg);
                    if res.is_err() {
                        break 'failed_conversation;
                    }
                }
                PAM_TEXT_INFO => {
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

                    PAM_BP_RENEW!(
                        &mut binary_prompt,
                        PAM_BP_RCONTROL!(&msgm[count].msg),
                        PAM_BP_LENGTH!(&msgm[count].msg)
                    );
                    PAM_BP_FILL!(
                        &mut binary_prompt,
                        0,
                        PAM_BP_LENGTH!(&msgm[count].msg),
                        PAM_BP_RDATA!(&msgm[count].msg)
                    );

                    if let Some(func) = pam_binary_handler_fn {
                        if func(appdata_ptr.clone(), &mut binary_prompt) != PAM_SUCCESS as i32
                            || binary_prompt.is_none()
                        {
                            break 'failed_conversation;
                        }
                    } else {
                        break 'failed_conversation;
                    }

                    if let Some(boxed_mystruct) = binary_prompt {
                        let mystruct = *boxed_mystruct;
                        string.push(mystruct.length.to_string());
                        string.push(mystruct.control.to_string());
                    } else {
                        string.clear();
                    }
                    //string = binary_prompt;
                    //binary_prompt = None;
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
                reply[count].resp.clone_from(&string);
                string.clear();
            }
            count += 1;
        } //loop

        *response = Some(reply);
        //reply = null;//离开作用域自动释放；
        return PAM_SUCCESS;
    } //end of goto,failed_conversation

    //D(the conversation failed);

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
                    //free reply[count].resp.clear(); rust会自动回收内存
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
        writeln!(io::stderr(), "{:?}", pam_misc_conv_die_line);
        pam_misc_conv_died = 1;
        return -1;
    }

    if pam_misc_conv_warn_time != 0 && now >= pam_misc_conv_warn_time {
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
