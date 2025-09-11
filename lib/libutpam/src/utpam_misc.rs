/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///一些辅助函数
use crate::common::{PAM_RETURN_VALUES, PAM_TOKEN_ACTIONS, PAM_TOKEN_RETURNS};
use crate::utpam::PAM_ACTION_UNDEF;
use std::mem::size_of;
//设置control_array数组的值并退出
macro_rules! set_and_break {
    ($array:expr) => {{
        for elem in $array.iter_mut() {
            *elem = -3;
        }
        return;
    }};
}

const DELIMITERS: &[char] = &[' ', '\t', '\n'];
//从给定的字符串中提取标记
pub fn utpam_tokenize<'a>(from: Option<&'a str>, next: &mut Option<&'a str>) -> Option<String> {
    let mut token = String::new();
    let from = match from {
        Some(s) => s,
        None => match next {
            Some(n) => *n,
            None => return None,
        },
    };

    // 找到第一个不在 DELIMITERS 中的字符，如果没找到返回字符长度
    let mut start = from
        .find(|c: char| !DELIMITERS.contains(&c))
        .unwrap_or(from.len());

    //字符串全是分隔符
    if start >= from.len() {
        *next = None;
        return None;
    }

    /*
     * 处理[...]
     * 将 [...] 中的内容作为一个整体，并去除方括号返回
     * 如果有嵌套的情况，则只处理最外层的[],即[..[..]..]这种情况最终返回..[..]..
     */
    let mut end = start;
    if from.chars().nth(start) == Some('[') {
        let mut to_escape = false;
        start += 1;

        for (i, c) in from.chars().enumerate().skip(start) {
            if c == '\\' {
                to_escape = true;
            } else if to_escape {
                to_escape = false;
            } else if c == ']' {
                token = from[start..i].to_string();
                end = i + 1;
                break;
            }
        }
    } else {
        // 查找下一个分隔符
        end = from[start..]
            .find(|c: char| DELIMITERS.contains(&c))
            .map_or(from.len(), |pos| start + pos);
        token = from[start..end].to_string();
    }

    // 更新 next
    if end < from.len() {
        *next = Some(&from[end..]);
    } else {
        *next = None;
    }

    Some(token.to_string())
}

//设置默认控制，即未指定的返回码使用默认动作
pub fn utpam_set_default_control(control_array: &mut [i32], default_action: i32) {
    for item in control_array.iter_mut().take(PAM_RETURN_VALUES) {
        if *item == PAM_ACTION_UNDEF {
            *item = default_action;
        }
    }
}

// 解析控制字符串tok，并更新control_array中的元素
pub fn utpam_parse_control(control_array: &mut [i32], mut tok: &str) {
    while !tok.is_empty() {
        // 去除空格，如果为空，则退出
        tok = tok.trim_start();
        if tok.is_empty() {
            break;
        }

        //遍历PAM_TOKEN_RETURNS数组，匹配返回码，并更新tok
        let mut ret = 0;
        for token in PAM_TOKEN_RETURNS.iter() {
            let len = token.len();
            if tok.starts_with(token) {
                tok = &tok[len..].trim_start();
                break;
            }
            ret += 1;
        }

        //如果没有匹配到返回码，或者tok为空，则退出
        if ret > PAM_RETURN_VALUES || tok.is_empty() {
            println!("expecting return values");
            set_and_break!(control_array);
        }

        // tok应该以'='开头，否则退出
        match tok.trim_start().chars().next() {
            Some(s) => {
                if s == '=' {
                    tok = &tok[1..].trim_start();
                } else {
                    println!("expecting '='");
                    set_and_break!(control_array);
                }
            }
            None => {
                println!("expecting action");
                set_and_break!(control_array);
            }
        }

        // 遍历PAM_TOKEN_ACTIONS数组，匹配动作，并更新tok
        let mut act: i32 = 0;
        for token in PAM_TOKEN_ACTIONS.iter() {
            let len = token.len();
            if tok.starts_with(token) {
                tok = &tok[len..].trim_start();
                break;
            }
            act += 1;
        }

        // 处理跳转
        if act > 0 {
            act = 0;
            for ch in tok.chars() {
                match ch.to_digit(10) {
                    Some(digit) => {
                        // 将字符串转换为数字，并检查是否溢出
                        match act
                            .checked_mul(10)
                            .and_then(|new_act| new_act.checked_add(digit as i32))
                        {
                            Some(new_act) => act = new_act,
                            None => {
                                println!("expecting smaller jump number");
                                set_and_break!(control_array);
                            }
                        }
                    }
                    None => {
                        break;
                    }
                };
                tok = &tok[1..].trim_start();
            }

            if act == 0 {
                println!("expecting non-zero");
                set_and_break!(control_array);
            }
        }

        //设置control_array元素
        if ret != PAM_RETURN_VALUES {
            control_array[ret] = act;
        } else {
            //将默认值设置为“act”
            utpam_set_default_control(control_array, act);
        }
    }
}

// 将输入字符串分割成参数
pub fn utpam_mkargv(input: &str, argv: &mut Vec<String>, argc: &mut i32) -> usize {
    let len = input.len();
    let mut total_size = 0;

    // 检查长度是否有效
    if len > 0 && len < usize::MAX / (size_of::<char>() + size_of::<String>()) {
        let mut tmp = None;
        let mut current = Some(input);

        while let Some(token) = utpam_tokenize(current, &mut tmp) {
            argv.push(token);

            // 检查 argc 是否溢出
            if *argc == i32::MAX {
                return total_size;
            }
            *argc += 1;

            current = tmp;
        }

        // 计算内存大小
        let total_len = argv.iter().map(|s| s.len()).sum::<usize>();
        let argv_len = argv.len();
        total_size = total_len + argv_len * size_of::<String>();
    }

    total_size
}
