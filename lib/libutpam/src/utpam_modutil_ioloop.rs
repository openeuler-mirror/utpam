/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::fs::File;
use std::io::{self, Read, Write};

/// 从给定的文件读取数据到缓冲区
pub fn utpam_modutil_read(file: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut total_bytes_read = 0; //记录已经读取的总字节数

    //只要还有未填满的缓冲区空间，就继续读取
    while total_bytes_read < buffer.len() {
        //从文件中读取数据到缓冲区的当前偏移位置。
        match file.read(&mut buffer[total_bytes_read..]) {
            Ok(0) => break, //到达文件末尾
            Ok(n) => total_bytes_read += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue, //读取被中断
            Err(e) => return Err(e),
        }
    }

    //返回读取的总字节数
    Ok(total_bytes_read)
}

/// 向给定的文件写入缓冲区中的数据
pub fn utpam_modutil_write(file: &mut File, buffer: &[u8]) -> io::Result<usize> {
    let mut total_bytes_written = 0;

    while total_bytes_written < buffer.len() {
        match file.write(&buffer[total_bytes_written..]) {
            Ok(0) => break, //对方关闭了连接或文件已满
            Ok(n) => total_bytes_written += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue, //写入被中断
            Err(e) => return Err(e),
        }
    }

    //返回写入的总字节数
    Ok(total_bytes_written)
}
