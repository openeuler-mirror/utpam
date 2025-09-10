use crate::utpam::CallSpi;
use libloading::{Library, Symbol};

//加载动态库
pub fn utpam_dlopen(path: String) -> Result<Library, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(path)?;
        Ok(lib)
    }
}

//从由 handle 指定的已加载库中查找名为 symbol 的符号
pub fn utpam_dlsym<'a>(
    handle: &'a Library,
    symbol: &'a [u8],
) -> Result<Symbol<'a, CallSpi>, Box<dyn std::error::Error>> {
    unsafe {
        let func: Symbol<CallSpi> = handle.get(symbol)?;
        Ok(func)
    }
}
