/**pam：
 * 延迟机制，应对认证过程中的攻击，尤其是那些试图通过快速、重复的认证尝试来猜测密码的攻击。
 * 这个机制的核心思想是在认证失败时引入随机延迟，这样可以增加攻击者的猜测成本，因为每次失败的认证请求都会导致一段不可预测的时间延迟。
 * 主要功能如下：
 *  重置定时器
 *  启动定时器
 *  计算延迟时间
 *  等待定时器
 *  设置延迟
 *
 * rust：利用 Rust 的并发模型来实现更高效和安全的延迟机制
 * 使用 rand 或者 rand_core 库来生成随机数
 *
 */
//use crate::utpam::*;
use crate::utpam::UtpamBoolean;
use std::any::Any;
use std::time::SystemTime;

pub type DelayFnPtr = Box<dyn Fn(i32, u64, Option<&dyn Any>) + Send + Sync>; //表示一个可以发送和同步的闭包
pub struct UtpamFailDelay {
    pub set: UtpamBoolean,
    pub delay: u64,
    pub begin: SystemTime,
    pub delay_fn_ptr: Option<DelayFnPtr>, // 使用Option来表示可选的延迟函数指针
}

impl UtpamFailDelay {
    ///构建一个新的UtpamFailDelay实例
    pub fn new() -> Self {
        UtpamFailDelay {
            set: UtpamBoolean::UtpamFalse,
            delay: 0,
            begin: SystemTime::now(),
            delay_fn_ptr: None,
        }
    }
}

impl Default for UtpamFailDelay {
    fn default() -> Self {
        Self::new()
    }
}
