use super::ReqMethod;
use crate::wx_pay::WxPay;

/// 支付的请求内容
#[derive(Debug)]
pub struct PayReq {
    pub method: ReqMethod,
    pub path: String,
}

/// 支付接口类别
#[derive(Debug)]
pub enum PayApi<'a> {
    Jsapi,
    App,
    WebH5,
    Native,
    GetTransactionsById { transaction_id: &'a str },
    // GetTransactionsByOutTradeNo,
}

impl PayApi<'_> {
    pub fn get_pay_req(&self, wx_pay: &WxPay) -> PayReq {
        match &self {
            PayApi::Jsapi => PayReq {
                method: ReqMethod::Post,
                path: "/v3/pay/transactions/jsapi".to_string(),
            },
            PayApi::App => PayReq {
                method: ReqMethod::Post,
                path: "/v3/pay/transactions/app".to_string(),
            },
            PayApi::WebH5 => PayReq {
                method: ReqMethod::Post,
                path: "/v3/pay/transactions/h5".to_string(),
            },
            PayApi::Native => PayReq {
                method: ReqMethod::Post,
                path: "/v3/pay/transactions/native".to_string(),
            },
            PayApi::GetTransactionsById { transaction_id } => PayReq {
                method: ReqMethod::Get,
                path: "/v3/pay/transactions/id/".to_string()
                    + transaction_id
                    + "?mchid="
                    + wx_pay.mchid,
            },
        }
    }
}
