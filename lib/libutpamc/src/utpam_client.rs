/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * This common file provides the prototypes for the UTPAM client API
 */
#[macro_export]
macro_rules! PAM_BP_ASSERT {
    ($x:expr) => {{
        println!("{:?}, {:?}: {:?}", file!(), line!(), $x);
    }
    #[cfg(feature = "ndebug")]
    {
        // 不做任何事情
    }};
}

//用于写入一个字节到指定的内存位置
#[macro_export]
macro_rules! __PAM_BP_WOCTET {
    ($data:expr, $offset:expr, $value:expr) => {{
        assert!(
            $offset < std::mem::size_of::<MyStruct>(),
            "Offset out of bounds"
        );
        let byte_index = $offset / std::mem::size_of::<u8>();
        let byte_offset = $offset % std::mem::size_of::<u8>();
        let mut bytes = unsafe {
            std::slice::from_raw_parts_mut(
                $data.as_mut() as *mut _ as *mut u8,
                std::mem::size_of::<MyStruct>(),
            )
        };
        bytes[byte_index] = $value;
    }};
}

//用于从指定的内存位置读取一个字节
#[macro_export]
macro_rules! __PAM_BP_ROCTET {
    ($x:expr, $y:expr) => {{
        // use bytemuck::cast_slice;
        let byte_slice = {
            let x_ptr = &$x as *const _ as *const u8;
            let size = std::mem::size_of_val($x);
            unsafe { std::slice::from_raw_parts(x_ptr, size) }
        };
        if $y >= byte_slice.len() {
            panic!("Index out of bounds");
        }
        byte_slice[$y]
    }};
}

//计算两个类型的大小之和
#[macro_export]
macro_rules! PAM_BP_MIN_SIZE {
    () => {
        (std::mem::size_of::<u32>() + std::mem::size_of::<u8>())
    };
}

// 最大长度常量
#[macro_export]
macro_rules! PAM_BP_MAX_LENGTH {
    () => {
        (0x20000)
    };
}

// 写入控制字节
#[macro_export]
macro_rules! PAM_BP_WCONTROL {
    ($x:expr, $value:expr) => {
        __PAM_BP_WOCTET!($x, 4, $value)
    };
}

// 读取控制字节
#[macro_export]
macro_rules! PAM_BP_RCONTROL {
    ($x:expr) => {
        __PAM_BP_ROCTET!($x, 4)
    };
}

// 计算总大小
#[macro_export]
macro_rules! PAM_BP_SIZE {
    ($x:expr) => {{
        (__PAM_BP_ROCTET!($x, 0) << 24)
            + (__PAM_BP_ROCTET!($x, 1) << 16)
            + (__PAM_BP_ROCTET!($x, 2) << 8)
            + (__PAM_BP_ROCTET!($x, 3))
    }};
}

// 计算数据长度
#[macro_export]
macro_rules! PAM_BP_LENGTH {
    ($x:expr) => {
        (PAM_BP_SIZE!($x) as usize - PAM_BP_MIN_SIZE!())
    };
}

//从指针 x 开始，跳过 PAM_BP_MIN_SIZE 个字节后的可变数据指针
#[macro_export]
macro_rules! PAM_BP_WDATA {
    ($x:expr) => {{
        let byte_slice = {
            let x_ptr = &$x as *const _ as *const u8;
            let size = std::mem::size_of_val($x);
            unsafe { std::slice::from_raw_parts_mut(x_ptr as *mut u8, size) }
        };
        let offset = PAM_BP_MIN_SIZE!();
        if offset > byte_slice.len() {
            panic!("Offset out of bounds");
        }
        &mut byte_slice[offset..]
    }};
}

//从指针 x 开始，跳过 PAM_BP_MIN_SIZE 个字节后的常量数据指针
#[macro_export]
macro_rules! PAM_BP_RDATA {
    ($x:expr) => {{
        let byte_slice = {
            let x_ptr = &$x as *const _ as *const u8;
            let size = std::mem::size_of_val($x);
            unsafe { std::slice::from_raw_parts(x_ptr, size) }
        };
        let offset = PAM_BP_MIN_SIZE!();
        if offset > byte_slice.len() {
            panic!("Offset out of bounds");
        }
        &byte_slice[offset..]
    }};
}

/* Note, this macro always '\0' terminates renewed packets */

#[macro_export]
macro_rules! PAM_BP_RENEW {
    ($old_p:expr, $cntrl:expr, $data_length:expr) => {{
        match $old_p {
            Some(ref mut old_p) => {
                if $cntrl != 0 {
                    let total_size = PAM_BP_MIN_SIZE!() + ($data_length);

                    // 创建一个新的向量来存储数据
                    let mut new_data = Box::new(MyStruct {
                        length: 0,
                        control: 0,
                    });
                    // 写入新的大小信息
                    __PAM_BP_WOCTET!(new_data, 0, (total_size >> 24) as u8);
                    __PAM_BP_WOCTET!(new_data, 1, (total_size >> 16) as u8);
                    __PAM_BP_WOCTET!(new_data, 2, (total_size >> 8) as u8);
                    __PAM_BP_WOCTET!(new_data, 3, total_size as u8);

                    // 设置控制字节
                    PAM_BP_WCONTROL!(new_data, $cntrl);

                    // 更新旧的数据
                    *old_p = new_data;
                } else {
                    // 如果控制字节为0，则设置为空向量
                    *old_p = Box::new(MyStruct {
                        length: 0,
                        control: 0,
                    });
                }
            }
            None => {
                PAM_BP_ASSERT!("programming error, invalid binary prompt pointer");
            }
        }
    }};
}

// 在一个二进制提示（binary prompt）的数据区域中填充指定长度的数据
#[macro_export]
macro_rules! PAM_BP_FILL {
    ($prompt:expr, $offset:expr, $length:expr, $data:expr) => {{
        let prompt = $prompt;
        let offset = $offset;
        let length = $length;
        let data = $data;

        let bp_length = PAM_BP_LENGTH!(prompt);
        if bp_length < (length + offset) {
            PAM_BP_ASSERT!("attempt to write over end of prompt");
        }

        let wdata = PAM_BP_WDATA!(prompt);
        wdata[offset..offset + length].copy_from_slice(&data[..length]);
    }};
}
