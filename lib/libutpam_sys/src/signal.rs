/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use nix::sys::signal;
use nix::sys::signal::{SigAction, Signal};

pub fn sigaction(signal_num: Signal, action: &SigAction) -> nix::Result<SigAction> {
    unsafe { signal::sigaction(signal_num, action) }
}
