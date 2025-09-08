/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
///一些辅助函数

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
