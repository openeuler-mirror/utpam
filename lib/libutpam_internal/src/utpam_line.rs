use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct UtpamLineBuffer {
    pub assembled: String,
    pub chunk: String,
}
impl UtpamLineBuffer {
    pub fn new() -> Self {
        UtpamLineBuffer {
            assembled: String::new(),
            chunk: String::new(),
        }
    }
}
impl Default for UtpamLineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

///处理配置文件内容
pub fn utpam_line_assemble(
    reader: &mut BufReader<File>,
    buffer: &mut UtpamLineBuffer,
    repl: String,
) -> i32 {
    for line_result in reader.lines() {
        let line = line_result.unwrap();

        // 跳过空行和注释行
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 去除行首和行尾的空白字符
        let line = line.trim().to_string();

        buffer.assembled.clear();

        // 如果以反斜杠结尾，则将反斜杠替换为空格，暂存在 buffer.chunk 中，直到遇到非反斜杠行
        if line.ends_with('\\') {
            let s = line.replace('\\', &repl);
            if !buffer.chunk.is_empty() {
                buffer.chunk.push_str(&s);
            } else {
                buffer.chunk = s;
            }
        } else {
            buffer.assembled = line;

            if !buffer.chunk.is_empty() {
                buffer.chunk.push_str(&buffer.assembled);
                buffer.assembled = buffer.chunk.clone();
                buffer.chunk.clear();
            }
        }
        if !buffer.assembled.is_empty() {
            return 1;
        }
    }

    // 在遍历结束时检查 buffer.chunk 是否还有内容
    if !buffer.chunk.is_empty() {
        buffer.assembled = buffer.chunk.clone();
        buffer.chunk.clear();
        return 1;
    }

    0
}
