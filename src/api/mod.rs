mod pay_api;
pub use pay_api::*;
mod data;
pub use data::*;

/// 请求类型
#[derive(Debug)]
pub enum ReqMethod {
    Get,
    Post,
}
impl ReqMethod {
    pub fn as_str(&self) -> &str {
        match &self {
            &ReqMethod::Get => "GET",
            &ReqMethod::Post => "POST",
        }
    }
}
