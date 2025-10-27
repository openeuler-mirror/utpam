/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::never_loop)]

use utpam::common::*;
use utpam::common::{UtpamMessage, UtpamResponse};
use utpam::utpam_overwrite_string;
use utpamc::*;

use lazy_static::lazy_static;
use nix::sys::signal::{self, SigAction, SigHandler, SigSet, Signal};
use nix::sys::signal::{sigprocmask, SigmaskHow};
use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg, Termios};
use nix::unistd::{alarm, isatty, read};
use std::any::Any;
use std::fmt;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

#[cfg(feature = "debug")]
use utpam::common::{utpam_output_debug, utpam_output_debug_info};
#[cfg(feature = "debug")]
use utpam::utpam_strerror::pam_strerror;
use utpam::D;

pub const NCCS: usize = 32;
pub const PAM_MISC_CONV_BUFSIZE: usize = 4096;
pub const ECHO: u8 = 10;
pub const CONV_ECHO_OFF: u8 = 0;
pub const CONV_ECHO_ON: u8 = 1;
const INPUTSIZE: usize = 1024;
const STDIN_FILENO: i32 = 0;
static PAM_MISC_CONV_WARN_LINE: &str = "...Time is running out...\n";
static PAM_MISC_CONV_DIE_LINE: &str = "...Sorry, your time is up!\n";

// 原子布尔变量，用于表示到期标志
static EXPIRED: AtomicBool = AtomicBool::new(false);
lazy_static! {
    static ref PAM_MISC_CONV_WARN_TIME: Mutex<u64> = Mutex::new(0); // 警告时间的时间戳
    static ref PAM_MISC_CONV_DIE_TIME: Mutex<u64> = Mutex::new(0); // 终止时间的时间戳
    static ref PAM_MISC_CONV_DIED: Mutex<i32> = Mutex::new(0); // 终止标志

    static ref PAM_BINARY_HANDLER_FN: Mutex<Option<PamBinaryHandlerFn>> = Mutex::new(None);
    static ref PAM_BINARY_HANDLER_Free: Mutex<Option<PamBinaryHandlerFree>> = Mutex::new(Some(pam_misc_conv_delete_binary));
}
type PamBinaryHandlerFn = fn(appdata: Option<Rc<dyn Any>>, prompt_p: &mut PamcBpT) -> i32;
type PamBinaryHandlerFree = fn(appdata: Option<Rc<dyn Any>>, prompt_p: PamcBpT);

type PamcBpT = Option<Box<MyStruct>>;
struct MyStruct {
    length: u32,
    control: u8,
}

//实现Display特性，用于将MyStruct类型转换为String类型
impl fmt::Display for MyStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.length, self.control as char)
    }
}
impl MyStruct {
    //将String类型转换为MyStruct类型
    fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').map(str::trim).collect();
        if parts.len() != 2 {
            return Err("Invalid format: expected two parts separated by a comma".to_string());
        }

        let length = parts[0]
            .parse::<u32>()
            .map_err(|_| "Failed to parse length as u32".to_string())?;
        let control_char = parts[1]
            .chars()
            .next()
            .ok_or_else(|| "Control part is empty".to_string())?;
        let control = control_char as u8;

        Ok(MyStruct { length, control })
    }
}

fn pam_misc_conv_delete_binary(
    _appdata: Option<Rc<dyn Any>>,
    mut delete_me: Option<Box<MyStruct>>,
) {
    PAM_BP_RENEW!(delete_me, 0, 0);
}

// 获取全局函数指针
macro_rules! get_fn {
    ($mutex:ident) => {
        *$mutex.lock().unwrap()
    };
}
// 设置全局函数指针，提供给外部使用
macro_rules! _set_fn {
    ($mutex:ident, $handler:expr) => {
        *$mutex.lock().unwrap() = Some($handler);
    };
}

//信号恢复，用于取消定时器并恢复之前的信号处理程序
fn reset_alarm(o_ptr: Option<SigAction>) {
    alarm::set(0);
    if let Some(o_ptr) = o_ptr {
        unsafe {
            signal::sigaction(Signal::SIGALRM, &o_ptr).expect("error");
        }
    }
}

//超时处理，更新EXPIRED值
extern "C" fn time_is_up(_: i32) {
    EXPIRED.store(true, Ordering::SeqCst);
}

//设置定时器，并注册一个新的信号处理函数
fn set_alarm(delay: u64, mut o_ptr: Option<SigAction>) -> i32 {
    //定义一个新的信号处理结构体
    let new_action = SigAction::new(
        SigHandler::Handler(time_is_up),
        signal::SaFlags::empty(),
        SigSet::empty(),
    );

    // 注册新的信号处理函数，并返回旧的信号处理程序
    let old_action = unsafe {
        match signal::sigaction(Signal::SIGALRM, &new_action) {
            Ok(old) => old,
            Err(_) => return 1,
        }
    };

    match alarm::set(delay as u32) {
        Some(_) => 0,
        None => {
            // 如果设置定时器失败，恢复之前的信号处理程序
            if let Some(ref mut o_ptr) = o_ptr {
                *o_ptr = old_action;
                if unsafe { signal::sigaction(Signal::SIGALRM, o_ptr) }.is_err() {
                    return 1;
                }
            };
            1
        }
    }
}

//用于计算距离下一次警报的时间间隔（以秒为单位）。通过检查当前时间与预设的警告时间和终止时间来决定返回值
fn get_delay() -> i32 {
    EXPIRED.store(false, Ordering::SeqCst); // 重置到期标志

    // 获取当前系统时间
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("时间获取失败")
        .as_secs();

    // 是否已过终止时间？
    {
        let die_time = PAM_MISC_CONV_DIE_TIME.lock().unwrap();
        let pam_misc_conv_die_time = *die_time;
        if pam_misc_conv_die_time != 0 && now >= pam_misc_conv_die_time {
            writeln!(io::stderr(), "{}", PAM_MISC_CONV_DIE_LINE).expect("写入 stderr 失败");

            let mut died_guard = PAM_MISC_CONV_DIED.lock().unwrap();
            *died_guard = 1; // 设置终止标志
            return -1;
        }
    }

    // 是否已过警告时间？
    let warn_time = PAM_MISC_CONV_WARN_TIME.lock().unwrap();
    let pam_misc_conv_warn_time = *warn_time;
    drop(warn_time); // 手动释放锁

    if pam_misc_conv_warn_time != 0 && now >= pam_misc_conv_warn_time {
        writeln!(io::stderr(), "{}", PAM_MISC_CONV_WARN_LINE).expect("写入 stderr 失败");

        let mut warn_t = PAM_MISC_CONV_WARN_TIME.lock().unwrap();
        *warn_t = 0; // 重置警告时间

        let die_time = PAM_MISC_CONV_DIE_TIME.lock().unwrap();
        let pam_misc_conv_die_time = *die_time;
        if pam_misc_conv_die_time != 0 {
            return (pam_misc_conv_die_time - now) as i32;
        } else {
            return 0;
        }
    }

    // 返回可能的警告延迟
    {
        let warn_time = PAM_MISC_CONV_WARN_TIME.lock().unwrap();
        let pam_misc_conv_warn_time = *warn_time;
        if pam_misc_conv_warn_time != 0 {
            return (pam_misc_conv_warn_time - now) as i32;
        }
    }
    {
        let die_time = PAM_MISC_CONV_DIE_TIME.lock().unwrap();
        let pam_misc_conv_die_time = *die_time;
        if pam_misc_conv_die_time != 0 {
            return (pam_misc_conv_die_time - now) as i32;
        }
    }

    0
}

//从标准输入读取用户输入，并根据传入的参数决定是否开启回显
fn read_string(echo: u8, prompt: String, retstr: &mut String) -> i32 {
    let stdin = io::stdin();
    let mut term_before: Option<Termios> = None;
    let mut term_tmp: Option<Termios> = None;
    let mut have_term = 0;
    let mut nset: SigSet;
    let mut oset = SigSet::empty();
    let old_sig: Option<SigAction> = None;
    let mut line = vec![0; INPUTSIZE - 1];
    let mut nc = 0;

    D!("called with echo={}, prompt={}.", echo, prompt);

    // 检查标准输入是否是终端
    if isatty(STDIN_FILENO).expect("REASON") {
        // 获取当前终端设置
        term_before = match tcgetattr(&stdin) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("Failed to get terminal attributes: {}", e);
                return -1;
            }
        };
        // 复制终端设置
        term_tmp.clone_from(&term_before);

        if echo == 0 {
            if let Some(ref mut t) = term_tmp {
                t.local_flags &= !LocalFlags::ECHO;
            }
        }
        have_term = 1;

        nset = SigSet::empty();
        nset.add(Signal::SIGTSTP);
        if let Err(e) = sigprocmask(SigmaskHow::SIG_BLOCK, Some(&nset), Some(&mut oset)) {
            eprintln!("Failed to block signals: {}", e);
            *retstr = String::new();
            return -1;
        }
    } else if echo == 0 {
        eprintln!("<warning: cannot turn echo off>");
        *retstr = String::new();
        return -1;
    }

    let mut delay = get_delay();

    while delay >= 0 {
        if have_term == 1 {
            if let Some(ref tmp) = term_tmp {
                if let Err(e) = tcsetattr(&stdin, SetArg::TCSAFLUSH, tmp) {
                    eprintln!("Failed to set terminal attributes: {}", e);
                    *retstr = String::new();
                    return -1;
                }
            }
        }

        writeln!(io::stderr(), "{}", prompt).expect("写入失败");

        if delay > 0 {
            if set_alarm(delay as u64, old_sig) == 1 {
                println!("<failed to set alarm>");
                break;
            }
        } else {
            if have_term == 1 {
                match read(STDIN_FILENO, &mut line) {
                    Ok(n) => {
                        nc = n;
                    }
                    Err(_) => {
                        if echo == 1 {
                            println!("\n");
                        }
                        *retstr = String::new();
                        if have_term == 1 {
                            sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oset), None).expect("msg");
                            if let Some(ref t) = term_before {
                                tcsetattr(&stdin, SetArg::TCSAFLUSH, t).expect("msg");
                            }
                        }
                        return nc.try_into().unwrap();
                    }
                }
            } else {
                nc = 0;
                while nc < INPUTSIZE - 1 && (if nc > 0 { line[nc - 1] } else { 0 }) != b'\n' {
                    match read(STDIN_FILENO, &mut line[nc..=nc]) {
                        Ok(rv) => {
                            if rv != 1 {
                                break;
                            }
                            nc += 1;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }

            if have_term == 1 {
                if let Some(ref t) = term_before {
                    if let Err(e) = tcsetattr(&stdin, SetArg::TCSADRAIN, t) {
                        eprintln!("Failed to set terminal attributes: {}", e);
                        *retstr = String::new();
                        return -1;
                    }
                }
                if echo == 0 || EXPIRED.load(Ordering::SeqCst) {
                    //不需要处理，C是换行，Rust这里自动换行了
                }

                if delay > 0 {
                    reset_alarm(old_sig);
                }
                if EXPIRED.load(Ordering::SeqCst) {
                    delay = get_delay();
                } else if nc > 0 {
                    if line[nc - 1] == b'\n' {
                        line[nc - 1] = 0;
                        nc -= 1;
                    } else {
                        if echo == 1 {
                            writeln!(io::stderr(), " ").expect("写入失败");
                        }
                        line[nc] = 0;
                    }

                    let retstr = String::from_utf8_lossy(&line[..nc]).to_string();
                    if retstr.is_empty() {
                        eprintln!("no memory for response string");
                    }

                    if have_term == 1 {
                        sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oset), None).expect("msg");
                        if let Some(ref t) = term_before {
                            tcsetattr(&stdin, SetArg::TCSAFLUSH, t).expect("msg");
                        }
                    }
                    return nc.try_into().unwrap();
                } else if nc == 0 {
                    println!("user did not want to type anything");
                    *retstr = String::new();
                    if echo == 1 {
                        writeln!(io::stderr(), " ").expect("写入失败");
                    }
                    if have_term == 1 {
                        sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oset), None).expect("msg");
                        if let Some(ref t) = term_before {
                            tcsetattr(&stdin, SetArg::TCSAFLUSH, t).expect("msg");
                        }
                    }
                    return nc.try_into().unwrap();
                }
            }
        }
    }

    println!("the timer appears to have expired");
    *retstr = String::new();

    if have_term == 1 {
        sigprocmask(SigmaskHow::SIG_SETMASK, Some(&oset), None).expect("msg");
        if let Some(ref t) = term_before {
            tcsetattr(&stdin, SetArg::TCSAFLUSH, t).expect("msg");
        }
    }
    nc.try_into().unwrap()
}

//回调函数，用于处理UTPAM的消息。根据消息类型调用 read_string 函数读取用户输入，并生成响应
#[no_mangle]
pub fn misc_conv(
    num_msg: usize,
    msgm: &[UtpamMessage],
    response: &mut Option<Vec<UtpamResponse>>,
    appdata_ptr: Option<Rc<dyn Any>>,
) -> u8 {
    let mut count: usize = 0;

    D!("allocating empty response structure array");

    let mut reply: Vec<UtpamResponse> = Vec::with_capacity(num_msg);

    D!("entering conversation function");

    'failed_conversation: loop {
        loop {
            if count >= num_msg {
                break;
            }

            let mut string: String = String::new();
            let nc: i32;

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
                    let mut binary_prompt: PamcBpT = None;

                    if msgm[count].msg.is_empty() || get_fn!(PAM_BINARY_HANDLER_FN).is_none() {
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

                    if let Some(func) = get_fn!(PAM_BINARY_HANDLER_FN) {
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
                        string = mystruct.to_string();
                    } else {
                        string.clear();
                    }
                }
                _ => {
                    writeln!(
                        io::stderr(),
                        "erroneous conversation ({})",
                        msgm[count].msg_style
                    )
                    .expect("写入 stderr 失败");
                    break 'failed_conversation;
                }
            }

            if !string.is_empty() {
                reply[count].resp_retcode = 0;
                reply[count].resp.clone_from(&string);
                string.clear();
            }
            count += 1;
        }

        *response = Some(reply);

        return PAM_SUCCESS;
    }

    D!("the conversation failed");

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
                    utpam_overwrite_string!(reply[count].resp);
                }
                PAM_BINARY_PROMPT => {
                    if let Some(func) = get_fn!(PAM_BINARY_HANDLER_Free) {
                        let bt_ptr = &reply[count].resp;
                        let parsed_struct =
                            MyStruct::from_string(bt_ptr).expect("Failed to parse string");
                        func(appdata_ptr.clone(), Some(Box::new(parsed_struct)));
                    }
                }
                PAM_ERROR_MSG | PAM_TEXT_INFO => {}
                _ => {}
            }

            reply[count].resp.clear();
            count += 1;
        }
    }
    PAM_CONV_ERR
}
