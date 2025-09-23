/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut, unused_variables)]
///存放libutpam的私有结构体和常量
use crate::common::{UtpamConv, UtpamXAuthData};
use crate::utpam_delay::UtpamFailDelay;
use crate::utpam_env::UtpamEnviron;

use libloading::Library;
use std::any::Any;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub const PAM_CALLED_FROM_MODULE: u8 = 1;
pub const PAM_CALLED_FROM_APP: u8 = 2;

pub const PAM_MT_DYNAMIC_MOD: u8 = 0;
pub const PAM_MT_STATIC_MOD: u8 = 1;
pub const PAM_MT_FAULTY_MOD: u8 = 2;

pub const PAM_NOT_STACKED: u8 = 0;
pub const PAM_AUTHENTICATE: u8 = 1;
pub const PAM_SETCRED: u8 = 2;
pub const PAM_ACCOUNT: u8 = 3;
pub const PAM_OPEN_SESSION: u8 = 4;
pub const PAM_CLOSE_SESSION: u8 = 5;
pub const PAM_CHAUTHTOK: u8 = 6;

pub const PAM_HT_MODULE: u8 = 0;
pub const PAM_HT_MUST_FAIL: u8 = 1;
pub const PAM_HT_SUBSTACK: u8 = 2;
pub const PAM_HT_SILENT_MODULE: u8 = 3;

pub const PAM_ACTION_IGNORE: i32 = 0;
pub const PAM_ACTION_OK: i32 = -1;
pub const PAM_ACTION_DONE: i32 = -2;
pub const PAM_ACTION_BAD: i32 = -3;
pub const PAM_ACTION_DIE: i32 = -4;
pub const PAM_ACTION_RESET: i32 = -5;
pub const PAM_ACTION_UNDEF: i32 = -6;

pub const PAM_SUBSTACK_MAX_LEVEL: i32 = 16;

pub const _PAM_INVALID_RETVAL: i8 = -1;

pub const PAM_ENV_CHUNK: u8 = 10;

pub const UTPAM_CONFIG: &str = "/etc/utpam.conf";
pub const UTPAM_CONFIG_D: &str = "/etc/utpam.d";
pub const UTPAM_CONFIG_DIST_D: &str = "/usr/lib/utpam.d";

pub const UTPAM_DEFAULT_SERVICE: &str = "other";

#[macro_export]
macro_rules! PAM_ACTION_IS_JUMP {
    ($x:expr) => {{
        if $x > 0 {
            true
        } else {
            false
        }
    }};
}

#[macro_export]
macro_rules! IF_NO_UTPAMH {
    ($expr:expr, $err:expr) => {{
        match $expr {
            Some(ref mut value) => value,
            None => return $err,
        }
    }};
}

#[macro_export]
macro_rules! UTPAM_FROM_MODULE {
    ($pamh:expr) => {
        if $pamh.caller_is == PAM_CALLED_FROM_MODULE {
            true
        } else {
            false
        }
    };
}
#[macro_export]
macro_rules! UTPAM_FROM_APP {
    ($pamh:expr) => {
        if $pamh.caller_is == PAM_CALLED_FROM_APP {
            true
        } else {
            false
        }
    };
}

#[macro_export]
macro_rules! UTPAM_TO_MODULE {
    ($pamh:expr) => {
        $pamh.caller_is = PAM_CALLED_FROM_MODULE
    };
}
#[macro_export]
macro_rules! UTPAM_TO_APP {
    ($pamh:expr) => {
        $pamh.caller_is = PAM_CALLED_FROM_APP
    };
}

#[derive(Debug)]
pub struct UtpamHandle {
    pub(super) authtok: String,
    pub(super) pam_conversation: Rc<UtpamConv>,
    pub(super) caller_is: u8,
    pub(super) oldauthtok: String,
    pub(super) prompt: Option<String>,
    pub(super) service_name: String,
    pub(super) user: String,
    pub(super) rhost: String,
    pub(super) ruser: String,
    pub(super) tty: String,
    pub(super) xdisplay: String,
    pub(super) authtok_type: String,
    pub(super) data: Option<Box<UtpamData>>,
    pub(super) env: Option<UtpamEnviron>,
    pub(super) fail_delay: UtpamFailDelay,
    pub(super) xauth: UtpamXAuthData,
    pub(super) handlers: Service,
    pub(super) former: UtpamFormerState,
    pub(super) mod_name: String,
    pub(super) mod_argc: i32,
    pub(super) mod_argv: Vec<String>,
    pub(super) choice: isize,
    pub(super) audit_state: isize,
    pub(super) authtok_verified: isize,
    pub(super) confdir: PathBuf,
}

impl UtpamHandle {
    pub fn new(
        service_name: String,
        pam_conversation: Rc<UtpamConv>,
        confdir: PathBuf,
        user: Option<String>,
    ) -> Self {
        UtpamHandle {
            authtok: String::default(),
            pam_conversation,
            caller_is: 0,
            oldauthtok: String::default(),
            prompt: None,
            service_name,
            user: String::default(),
            rhost: String::default(),
            ruser: String::default(),
            tty: String::default(),
            xdisplay: String::default(),
            authtok_type: String::default(),
            data: None,
            env: None,
            fail_delay: UtpamFailDelay::default(),
            xauth: UtpamXAuthData {
                namelen: 0,
                name: None,
                datalen: 0,
                data: vec![],
            },
            handlers: Service {
                module: vec![],
                modules_allocated: 0,
                modules_used: 0,
                handlers_loaded: 0,
                conf: Handlers {
                    authenticate: None,
                    setcred: None,
                    acct_mgmt: None,
                    open_session: None,
                    close_session: None,
                    chauthtok: None,
                },
                other: Handlers {
                    authenticate: None,
                    setcred: None,
                    acct_mgmt: None,
                    open_session: None,
                    close_session: None,
                    chauthtok: None,
                },
            },
            former: UtpamFormerState {
                choice: 0,
                depth: 0,
                impression: 0,
                status: 0,
                substates: vec![],
                fail_user: 0,
                want_user: UtpamBoolean::UtpamFalse,
                prompt: None,
                update: UtpamBoolean::UtpamFalse,
            },
            mod_name: String::default(),
            mod_argc: 0,
            mod_argv: vec![],
            choice: 0,
            audit_state: 0,
            authtok_verified: 0,
            confdir,
        }
    }

    //    /// 公共方法，允许其他包获取字段的引用。
    //     pub fn get_field(&self) -> &str {
    //         &self.field
    //     }

    //     /// 公共方法，允许其他包修改字段。
    //     pub fn set_field(&mut self, field: String) {
    //         self.field = field;
    //     }
}

pub type CleanupFn = fn(&mut UtpamHandle, Option<Rc<dyn Any>>, i32);

#[derive(Debug, Clone)]
pub struct UtpamData {
    pub(super) name: String,
    pub(super) data: Option<Rc<dyn Any>>, //存放任意类型数据
    pub(super) cleanup: Option<CleanupFn>,
    pub(super) next: Option<Box<UtpamData>>,
}

#[derive(Debug, Clone)]
pub enum UtpamBoolean {
    UtpamFalse,
    UtpamTrue,
}

impl UtpamBoolean {
    pub fn to_bool(&self) -> bool {
        match self {
            UtpamBoolean::UtpamTrue => true,
            UtpamBoolean::UtpamFalse => false,
        }
    }
}

#[derive(Debug)]
pub struct Service {
    pub(super) module: Vec<LoadedModule>,
    pub(super) modules_allocated: isize,
    pub(super) modules_used: i32,
    pub(super) handlers_loaded: isize,
    pub(super) conf: Handlers,
    pub(super) other: Handlers,
}

#[derive(Debug)]
pub struct LoadedModule {
    pub(super) name: String,
    pub(super) moule_type: u8,
    pub(super) dl_handle: Option<Library>,
}

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(super) authenticate: Option<Box<Handler>>,
    pub(super) setcred: Option<Box<Handler>>,
    pub(super) acct_mgmt: Option<Box<Handler>>,
    pub(super) open_session: Option<Box<Handler>>,
    pub(super) close_session: Option<Box<Handler>>,
    pub(super) chauthtok: Option<Box<Handler>>,
}

//定义类型别名CallSpi
pub type CallSpi = fn(
    utpamh: &mut Box<UtpamHandle>,
    flags: u32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> u8;

#[derive(Debug, Clone)]
pub struct Handler {
    pub(super) handler_type: u8,
    pub(super) func: Option<CallSpi>,
    pub(super) actions: Vec<i32>,
    pub(super) cached_retval: Rc<RefCell<i8>>, //用于实现内部可变性和共享所有权
    pub(super) argc: i32,
    pub(super) argv: Vec<String>,
    pub(super) next: Option<Box<Handler>>,
    pub(super) mod_name: String,
    pub(super) stack_level: i32,
    pub(super) grantor: isize,
}

impl Handler {
    //设置cached_retval 字段的值
    pub fn set_cached_retval(&self, value: i8) {
        *self.cached_retval.borrow_mut() = value;
    }
    //获取cached_retval 字段的值
    pub fn get_cached_retval(&self) -> i8 {
        *self.cached_retval.borrow()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UtpamSubstackState {
    pub(super) impression: i32,
    pub(super) status: u8,
}

#[derive(Debug, Clone)]
pub struct UtpamFormerState {
    pub(super) choice: u8,
    pub(super) depth: i32,
    pub(super) impression: i32,
    pub(super) status: u8,
    pub(super) substates: Vec<UtpamSubstackState>,
    pub(super) fail_user: u8,
    pub(super) want_user: UtpamBoolean,
    pub(super) prompt: Option<String>,
    pub(super) update: UtpamBoolean,
}
