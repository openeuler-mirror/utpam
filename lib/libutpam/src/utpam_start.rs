#![allow(dead_code, unused_mut)]
#![allow(unused_variables)]
use crate::common::*;
use crate::parse::*;
use crate::utpam::UtpamHandle;
use crate::utpam_handlers::*;

pub fn utpam_start(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    utpam_start_internal(service_name, user, utpam_conversation, None, utpamh)
}

pub fn utpam_stat_confdir(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: Option<String>,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    utpam_start_internal(service_name, user, utpam_conversation, confdir, utpamh)
}

fn utpam_start_internal(
    service_name: String,
    user: Option<String>,
    utpam_conversation: UtpamConv,
    confdir: Option<String>,
    mut utpamh: &mut Option<Box<UtpamHandle>>,
) -> i32 {
    //处理服务名称
    let service_name = parse_str(service_name);
    let mut pamh = Box::new(UtpamHandle::new(service_name, utpam_conversation, user));

    //实例化UtpamHandle
    if utpam_init_handlers(&mut pamh) != PAM_SUCCESS {
        //日志处理（待补充）
        return PAM_ABORT;
    }

    //返回初始化好的UtpamHandle结构体
    *utpamh = Some(pamh);
    PAM_SUCCESS
}
