use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, KeyInit, Payload},
    Aes256Gcm,
};
use base64::{engine, Engine};
use pkcs8::DecodePrivateKey;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Error;
use rsa::{
    sha2::{Digest, Sha256},
    Pkcs1v15Sign, RsaPrivateKey,
};
use serde::{Deserialize, Serialize};

// use crate::random::rand_string;
// use crate::utils::get_slice_arr;

use crate::{
    api::{Amount, Jsapi, PayApi, Payer, WxData},
    fetch::post,
    utils::{gen_rand_str, get_timestamp, rsa_sign},
};

pub struct WxPay<'a> {
    /// 【公众号ID】 公众号ID
    pub appid: &'a str,
    /// 【直连商户号】 直连商户号
    pub mchid: &'a str,
    pub private_key: &'a str,
    pub serial_no: &'a str,
    pub api_v3_private_key: &'a str,
    /// 【通知地址】 异步接收微信支付结果通知的回调地址，通知URL必须为外网可访问的URL，不能携带参数。 公网域名必须为HTTPS，如果是走专线接入，使用专线NAT IP或者私有回调域名可使用HTTP
    pub notify_url: &'a str,
    pub certificates: Option<&'a str>,
}

impl<'a> WxPay<'a> {
    /// jsapi 支付，返回客户端的支付参数信息
    pub async fn jsapi(&self, body: &Jsapi) -> anyhow::Result<WxData> {
        let pay_api = PayApi::Jsapi;
        let pay_req = pay_api.get_pay_req(&self);
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct JsapiData {
            description: String,
            out_trade_no: String,
            amount: Amount,
            payer: Payer,
            appid: String,
            mchid: String,
            notify_url: String,
        }
        let jsapi_params = JsapiData {
            description: body.description.clone(),
            out_trade_no: body.out_trade_no.clone(),
            amount: body.amount.clone(),
            payer: body.payer.clone(),
            appid: self.appid.to_string(),
            mchid: self.mchid.to_string(),
            notify_url: self.notify_url.to_string(),
        };
        #[derive(Serialize, Deserialize, Debug)]
        struct JsapiRes {
            /// 【预支付交易会话标识】 预支付交易会话标识。用于后续接口调用中使用，该值有效期为2小时
            pub prepay_id: String,
        }
        let pre_data: JsapiRes = post(&self, &pay_req, &jsapi_params).await?;
        let pack = "prepay_id=".to_string() + pre_data.prepay_id.as_str();
        let ran_str = gen_rand_str();
        let now_time = get_timestamp();
        // 获取签名
        let pay_sign = rsa_sign(
            &self.private_key,
            self.appid.to_string()
                + "\n"
                + now_time.as_str()
                + "\n"
                + ran_str.as_str()
                + "\n"
                + pack.as_str()
                + "\n",
        )?;
        Ok(WxData {
            sign_type: "RSA".into(),
            pay_sign,
            package: pack,
            nonce_str: ran_str,
            time_stamp: now_time.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::WxPay;
    use chrono::Local;
    use uuid::Uuid;

    #[test]
    fn test_time() {
        let dt = Local::now();
        println!("dddd33, {:?}", dt);
        let timestamp = dt.timestamp();
        println!("timsss23  {}", timestamp);

        let id = Uuid::new_v4().to_string().replace("-", "");

        println!("idid  {}", id);
        println!("idid  {}", id.len());
    }
}
