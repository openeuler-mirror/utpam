/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///一些辅助函数

const DELIMITERS: &[char] = &[' ', '\t', '\n'];
//从给定的字符串中提取标记
pub fn utpam_tokenize<'a>(from: Option<&'a str>, next: &mut Option<&'a str>) -> Option<String> {
    let from = match from {
        Some(s) => s,
        None => match next {
            Some(n) => *n,
            None => return None,
        },
    };

    // 找到第一个不在 DELIMITERS 中的字符，如果没找到返回字符长度
    let start = from
        .find(|c: char| !DELIMITERS.contains(&c))
        .unwrap_or(from.len());

    //字符串全是分隔符
    if start >= from.len() {
        *next = None;
        return None;
    }

    //处理[...]
    let mut end = start;
    if from.chars().nth(start) == Some('[') {
        let mut to_escape = false;
        let mut depth = 1; //[...]嵌套层数
        for (i, c) in from.chars().enumerate().skip(start + 1) {
            if c == '\\' {
                to_escape = true;
            } else if to_escape {
                to_escape = false;
            } else if c == ']' {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            } else if c == '[' {
                depth += 1; //[..[..]..] 增加嵌套层数
            }
        }
    } else {
        // 查找下一个分隔符
        end = from[start..]
            .find(|c: char| DELIMITERS.contains(&c))
            .map_or(from.len(), |pos| start + pos);
    }

    //从 from 中提取两个分隔符之间的子串
    let token = from[start..end].to_string();

    // 更新 next
    if end < from.len() {
        *next = Some(&from[end..]);
    } else {
        *next = None;
    }

    Some(token)
}
