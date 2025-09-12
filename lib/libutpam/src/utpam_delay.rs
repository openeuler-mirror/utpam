/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/**
 * 延迟机制，应对认证过程中的攻击，尤其是那些试图通过快速、重复的认证尝试来猜测密码的攻击。
 * 这个机制的核心思想是在认证失败时引入随机延迟，这样可以增加攻击者的猜测成本，因为每次失败的认证请求都会导致一段不可预测的时间延迟。
 *
 */
use crate::common::PAM_SUCCESS;
use crate::utpam::UtpamBoolean;
use crate::utpam::UtpamHandle;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::any::Any;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

//pub type DelayFnPtr = Box<dyn Fn(i32, u32, Option<&dyn Any>) + Send + Sync>; //表示一个可以发送和同步的闭包
pub type DelayFnPtr = fn(i32, u32, Option<&dyn Any>) -> u64;

#[derive(Debug, Clone)]
pub struct UtpamFailDelay {
    pub set: UtpamBoolean,
    pub delay: u64,
    pub begin: SystemTime,
    pub delay_fn_ptr: Option<DelayFnPtr>,
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

    ///重置延迟状态
    pub fn utpam_reset_timer(&mut self) {
        self.set = UtpamBoolean::UtpamFalse;
    }

    /// 启动定时器（Rust要求字段不能为空，是不是这部分内容不需要格外实现）
    pub fn utpam_start_timer(&mut self) {
        self.begin = SystemTime::now();
    }

    /// 计算延迟时间，延迟时间的范围在基础值的 50% 到 150%之间
    pub fn utpam_compute_delay(&self, seed: u64, base: u64) -> u64 {
        let mut rng = StdRng::seed_from_u64(seed); //基于seed生成随机数

        let mut sum: f64 = 0.0;
        for _ in 0..3 {
            let seed = rng.gen::<u64>();
            sum += (seed / 10) as f64 % 1_000_000.0; //限制seed范围在 0 到 999999 之间
        }
        let avg = (sum / 3.0) / 1e6 - 0.5;
        let delay = base as f64 * (1.0 + avg);
        if delay > u64::MAX as f64 {
            u64::MAX
        } else {
            delay as u64
        }
    }
    ///记录和更新最大延迟
    pub fn utpam_set_delay(&mut self, usec: u64) {
        if !self.set.to_bool() {
            self.set = UtpamBoolean::UtpamTrue;
            self.delay = 0;
        }

        //比较传入的 usec 和当前的最大延迟时间 largest
        if self.delay < usec {
            self.delay = usec;
        }
    }
}

impl Default for UtpamFailDelay {
    fn default() -> Self {
        Self::new()
    }
}

/// 根据计算出的延迟时间，执行等待操作
pub fn utpam_await_timer(utpamh: &mut Box<UtpamHandle>, status: i32) {
    let fail_delay = &mut utpamh.fail_delay;

    // 将当前时间转换为从 Unix 纪元开始的秒数
    let current_time = match fail_delay.begin.duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => {
            //日记记录
            println!("Failed to get duration since Unix epoch.");
            return;
        }
    };

    // 计算延迟时间
    let delay = fail_delay.utpam_compute_delay(current_time, fail_delay.delay);

    if let Some(ref delay_fn) = fail_delay.delay_fn_ptr {
        let delay_u = if delay > u32::MAX as u64 {
            u32::MAX
        } else {
            delay as u32
        };

        if let Some(appdata_ptr) = &utpamh.pam_conversation.appdata_ptr {
            delay_fn(status, delay_u, Some(appdata_ptr));
        } else {
            delay_fn(status, delay_u, None);
        }
    } else if status != PAM_SUCCESS && fail_delay.set.to_bool() {
        //如果认证失败设置延迟
        if delay > 0 {
            thread::sleep(Duration::from_micros(delay)); //延迟dalay微秒
        }
    }
    fail_delay.utpam_reset_timer();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_delay() {
        let fail_delay = UtpamFailDelay::new();
        let delay = fail_delay.utpam_compute_delay(10, 100);
        assert!(delay >= 50 && delay <= 150);
    }
}
