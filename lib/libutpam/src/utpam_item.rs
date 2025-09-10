///设置和获取UtpamHandle结构体字段文件
use users::get_current_username;

//获取当前用户名
pub fn get_username() -> String {
    match get_current_username() {
        Some(username) => username.to_string_lossy().into_owned(),
        None => {
            "login:".to_string() // 如果获取用户名失败，返回默认值
        }
    }
}
