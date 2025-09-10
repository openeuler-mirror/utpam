///处理字符串

///去除最后一个斜杠前面的字符串，返回剩余的字符串，并转换为小写
pub fn parse_str(s: String) -> String {
    let last_slash_pos = s.rfind('/');
    let last_part = if let Some(pos) = last_slash_pos {
        &s[pos + 1..]
    } else {
        &s
    };
    last_part.to_lowercase()
}
