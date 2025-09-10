/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code, unused_mut, unused_variables)]
///存放libutpam的私有结构体和常量
use crate::common::{UtpamConv, UtpamXAuthData, PAM_RETURN_VALUES};
use crate::utpam_delay::UtpamFailDelay;
use crate::utpam_env::UtpamEnviron;

use libloading::Library;
use std::path::PathBuf;

pub const PAM_MT_DYNAMIC_MOD: i32 = 0;
pub const PAM_MT_STATIC_MOD: i32 = 1;
pub const PAM_MT_FAULTY_MOD: i32 = 2;

pub const PAM_NOT_STACKED: i32 = 0;
pub const PAM_AUTHENTICATE: i32 = 1;

pub const PAM_HT_MODULE: i32 = 0;
pub const PAM_HT_MUST_FAIL: i32 = 1;
pub const PAM_HT_SUBSTACK: i32 = 2;
pub const PAM_HT_SILENT_MODULE: i32 = 3;

pub const PAM_ACTION_IGNORE: i32 = 0;
pub const PAM_ACTION_OK: i32 = -1;
pub const PAM_ACTION_DONE: i32 = -2;
pub const PAM_ACTION_BAD: i32 = -3;
pub const PAM_ACTION_DIE: i32 = -4;
pub const PAM_ACTION_RESET: i32 = -5;
pub const PAM_ACTION_UNDEF: i32 = -6;

pub const PAM_SUBSTACK_MAX_LEVEL: i32 = 16;

pub const UTPAM_CONFIG: &str = "/etc/utpam.conf";
pub const UTPAM_CONFIG_D: &str = "/etc/utpam.d";
pub const UTPAM_CONFIG_DIST_D: &str = "/usr/lib/utpam.d";

pub const UTPAM_DEFAULT_SERVICE: &str = "other";

pub struct UtpamHandle {
    pub(super) authtok: String,
    pub(super) pam_conversation: UtpamConv,
    pub caller_is: u32,
    pub(super) oldauthtok: String,
    pub(super) prompt: String,
    pub service_name: String,
    pub(super) user: String, //可以为空，修改：String -> Option<String>
    pub(super) rhost: String,
    pub(super) ruser: String,
    pub(super) tty: String,
    pub(super) xdisplay: String,
    pub(super) authtok_type: String,
    pub(super) data: UtpamData,
    pub(super) env: UtpamEnviron,
    pub(super) fail_delay: UtpamFailDelay,
    pub(super) xauth: UtpamXAuthData,
    pub(super) handlers: Service,
    pub(super) former: UtpamFormerState,
    pub(super) mod_name: String,
    pub(super) mod_argc: isize,
    pub(super) mod_argv: Vec<String>,
    pub(super) choice: isize,
    pub(super) audit_state: isize,
    pub(super) authtok_verified: isize,
    pub(super) confdir: PathBuf,
}

impl UtpamHandle {
    pub fn new(
        service_name: String,
        pam_conversation: UtpamConv,
        confdir: PathBuf,
        user: Option<String>,
    ) -> Self {
        UtpamHandle {
            authtok: String::default(),
            pam_conversation,
            caller_is: 12,
            oldauthtok: String::default(),
            prompt: String::default(),
            service_name,
            user: String::default(),
            rhost: String::default(),
            ruser: String::default(),
            tty: String::default(),
            xdisplay: String::default(),
            authtok_type: String::default(),
            data: UtpamData {
                name: None,
                data: Box::new(()),
                cleanup: None,
                next: None,
            },
            env: UtpamEnviron::default(),
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
                want_user: 0,
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

//定义一个trait来模拟清理函数的行为
trait CleanupFn {
    fn cleanup(&self, utpamh: &mut Option<Box<UtpamHandle>>, data: Vec<String>, error_status: i32);
}

pub struct UtpamData {
    name: Option<String>,
    data: Box<dyn std::any::Any>, // 表示可以存储任何类型的数据。Box<dyn Any> 是一个智能指针，它指向一个实现了 Any trait 的值。
    cleanup: Option<Box<dyn CleanupFn>>,
    next: Option<Box<UtpamData>>, //待定，是否考虑使用Option<Rc<RefCell<PamData>>>
}

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

pub struct Service {
    pub(super) module: Vec<LoadedModule>,
    pub(super) modules_allocated: isize,
    pub(super) modules_used: i32,
    pub(super) handlers_loaded: isize,
    pub(super) conf: Handlers,
    pub(super) other: Handlers,
}

pub struct LoadedModule {
    pub(super) name: String,
    pub(super) moule_type: i32,
    pub(super) dl_handle: Option<Library>,
}

#[derive(Clone)]
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
    utpamh: &mut Option<Box<UtpamHandle>>,
    flags: i32,
    argc: Option<i32>,
    argv: Option<Vec<String>>,
) -> i32;

#[derive(Clone)]
pub struct Handler {
    pub(super) handler_type: isize,
    pub(super) cleanup: CallSpi,
    pub(super) actions: [isize; PAM_RETURN_VALUES],
    pub(super) cached_retval: isize,
    pub(super) cached_retval_p: Option<*mut isize>,
    pub(super) argc: isize,
    pub(super) argv: Vec<String>,
    pub(super) next: Option<Box<Handler>>,
    pub(super) mod_name: String,
    pub(super) stack_level: isize,
    pub(super) grantor: isize,
}

struct UtpamSubstackState {
    impression: isize,
    status: isize,
}
pub struct UtpamFormerState {
    pub(super) choice: i32,
    depth: isize,
    impression: isize,
    status: isize,
    substates: Vec<UtpamSubstackState>,
    fail_user: isize,
    want_user: isize,
    prompt: Option<String>,
    update: UtpamBoolean,
}
