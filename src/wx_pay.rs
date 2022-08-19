

use reqwest::header::{CONTENT_TYPE, ACCEPT, HeaderMap, AUTHORIZATION, USER_AGENT};
use reqwest::{Error, Response};
use serde::{Serialize, Deserialize};
use serde_json::value::Value;

use rsa::{RsaPrivateKey, PaddingScheme, Hash, pkcs8::DecodePrivateKey};
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use std::iter::repeat;

use crate::rand_string;
use chrono::Utc;
use aes_gcm::{
    aead::{Aead, KeyInit, generic_array::GenericArray, Payload},
    Aes256Gcm, Nonce
};

pub struct WxPay<'a> {
    pub appid: &'a str,
    pub mchid: &'a str,
    pub private_key: &'a str,
    pub serial_no: &'a str,
    pub apiv3_private_key: &'a str,
    pub notify_url: &'a str,
    pub certificates: Option<&'a str>,
}

#[derive(Serialize, Debug)]
pub struct WxData {
    pub sign_type: String,
    pub pay_sign: String,
    pub package: String,
    pub nonce_str: String,
    pub time_stamp: String
}


#[derive(Serialize, Deserialize)]
pub struct Amount {
    pub total: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Payer {
    pub openid: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsapiParams {
    pub description: String,
    pub out_trade_no: String,
    pub amount: Amount,
    pub payer: Payer,
}

#[derive(Clone, Copy)]
struct ApiBody<'a> {
    url: &'a str,
    method: Method,
    pathname: &'a str,
}
#[derive(Clone, Copy)]
enum Method {
    GET,
    POST,  
} 



impl<'a> WxPay<'a> {
    fn rsa_sign(&self, content: String, private_key: &str) -> String {
        // let der_bytes = base64::decode(der_encoded).expect("Failed to decode base64 content");
        // 获取私钥对象
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key).expect("Failed to parse key");
        // 创建一个Sha256对象
        let mut hasher = Sha256::new();
        // 对内容进行摘要
        hasher.input_str(content.as_str());
        // 将摘要结果保存到buf中
        let mut buf: Vec<u8> = repeat(0).take((hasher.output_bits() + 7) / 8).collect();
        hasher.result(&mut buf);
        // 对摘要进行签名
        let sign_result = private_key.sign(
            PaddingScheme::PKCS1v15Sign { hash: Option::from(Hash::SHA2_256) },
            &buf
        );
        // 签名结果转化为 base64.
        let vec = sign_result.expect("Create sign error for base64");
        base64::encode(vec)
    }

    fn get_headers(&self, api_body:ApiBody, params_string: String) -> Result<HeaderMap, Error> {
        let dt = Utc::now();
        let timestamp = dt.timestamp();
        let onece_str = rand_string(32);
        let method = match api_body.method {
            Method::GET => "GET",
            Method::POST => "POST"
        };
        // 获取签名
        let signature = self.rsa_sign(
            method.to_string() + "\n"
                + api_body.pathname + "\n" 
                + timestamp.to_string().as_str() + "\n"
                + onece_str.as_str() + "\n"
                + params_string.as_str() + "\n"
            , 
            &self.private_key
        );
        // 组装header
        let authorization = "WECHATPAY2-SHA256-RSA2048 mchid=\"".to_string()
            + &self.mchid + "\",nonce_str=\""
            + onece_str.as_str() + "\",timestamp=\""
            + timestamp.to_string().as_str() + "\",signature=\""
            + signature.as_str() + "\",serial_no=\""
            + &self.serial_no + "\"";
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json; charset=utf-8".parse().unwrap());
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(AUTHORIZATION, authorization.parse().unwrap());
        headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.5060.134 Safari/537.36 Edg/103.0.1264.71".parse().unwrap());

        Ok(headers)

    }
    
    /// ### jsapi 微信支付
    /// 
    /// 使用示例：
    /// ```
    /// use wx_pay::WxPay;
    /// 
    ///     ...
    ///     // 初始化 
    ///     let wx_pay = WxPay {
    ///         appid: WECHAT_MINI_APP_ID,
    ///         mchid: WECHAT_PAY_MCH_ID,
    ///         private_key: WECHAT_PRIVATE_KEY,
    ///         serial_no: WECHAT_PAY_SERIAL,
    ///         apiv3_private_key: WECHAT_PAY_APIV3,
    ///         notify_url: WECHAT_PAY_NOTIFY_URL,  // 支付回调地址
    ///         certificates: None
    ///     };
    /// 
    ///     let data = wx_pay.jsapi(JsapiParams {
    ///         description: "测试122".to_string(),
    ///         out_trade_no: rand_string(16),  // 随机字符串
    ///         amount: Amount { total: 1 },
    ///         payer: Payer { openid: openid}
    ///     }).await.unwrap();
    /// 
    /// 
    /// ```
    pub async fn jsapi(&self, params: JsapiParams) -> Result<WxData, Error> {

        #[derive(Serialize, Deserialize)]
        struct Jsapi<'a> {
            description: String,
            out_trade_no: String,
            amount: Amount,
            payer: Payer,
            appid: &'a str,
            mchid: &'a str,
            notify_url: &'a str,
        }
        let jsapi_params = Jsapi {
            description: params.description,
            out_trade_no: params.out_trade_no,
            amount: params.amount,
            payer: params.payer,
            appid: &self.appid,
            mchid: &self.mchid,
            notify_url: &self.notify_url,
        };

        let jsapi_str = serde_json::to_string(&jsapi_params).unwrap();

        let api_body = ApiBody {
            url: "https://api.mch.weixin.qq.com/v3/pay/transactions/jsapi",
            method: Method::POST,
            pathname: "/v3/pay/transactions/jsapi",
        };

        let headers_all = self.get_headers(api_body, jsapi_str).unwrap();

        #[derive(Serialize, Deserialize, Debug)]
        struct JsapiRes {
            prepay_id: String
        }
        let client = reqwest::Client::new();
        let pre_data: JsapiRes = client.post(api_body.url.clone())
            .headers(headers_all)
            .json(&jsapi_params)
            .send()
            .await
            .unwrap()
            .json()
            .await.unwrap();

        let ran_str = rand_string(32);
        let pack = "prepay_id=".to_string() + pre_data.prepay_id.as_str();
        let dt = Utc::now();
        let now_time = dt.timestamp();
        
        // 获取签名
        let pay_si = self.rsa_sign(
            self.appid.to_string() + "\n"
                + now_time.to_string().as_str() + "\n" 
                + ran_str.as_str() + "\n"
                + pack.as_str() + "\n"
            ,
            &self.private_key
        );

        let wx_data = WxData {
            sign_type: "RSA".into(),
            pay_sign: pay_si,
            package: pack,
            nonce_str: ran_str,
            time_stamp: now_time.to_string(),
        };
        
        Ok(wx_data)
    }


}






#[derive(Serialize, Deserialize, Debug)]
pub struct WxDecodeData {
    pub mchid: String,
    pub appid: String,
    pub out_trade_no: String,
    pub transaction_id: String,
    pub trade_type: String,
    pub trade_state: String,
    pub trade_state_desc: String,
    pub bank_type: String,
    pub attach: String,
    pub success_time: String,
    pub payer: Payer,
    pub amount: WxDecodeDataAmount
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WxDecodeDataAmount {
    pub total: u32,
    pub payer_total: u32,
    pub currency: String,
    pub payer_currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct  WxPayNotifyResource {
    pub algorithm: String,
    pub associated_data: String,
    pub ciphertext: String,
    pub nonce: String,
    pub original_type: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WxPayNotify {
    pub create_time: String,
    pub event_type: String,
    pub id: String,
    pub resource: WxPayNotifyResource,
    pub resource_type: String,
    pub summary: String,
}


/// ## 微信支付，解密
/// 
/// wechat_pay_apiv3 为apiv3 密钥
/// params 为微信回调请求数据
/// 
/// 使用示例 (actix-web为例，回调接口)
/// 
/// ```
/// use wx_pay::{decode_wx, WxPayNotify};
/// 
/// #[post("/pay/notify_url/action")]
/// pub async fn pay_notify_url_action(params: web::Json<WxPayNotify>) -> Result<impl Responder> {
///     println!("##############  微信支付 回调 #############");
///     println!("{:#?}", params);
///     let t_params = params.0;
///     let data = decode_wx(t_params).unwrap();
///     println!("json  {:#?}", data);
///     println!("##############  微信支付 回调end #############");
///     Ok(web::Json(ResultStatus {status: 2, message: "成功".into()}))
/// }
/// 
/// ```
/// 
/// 
pub fn decode_wx(wechat_pay_apiv3: &str, params: WxPayNotify) -> Result<WxDecodeData, Error> {
    let auth_key_length = 16;

    let mut t_key = [0u8; 32];
    hex::decode_to_slice(hex::encode(wechat_pay_apiv3), &mut t_key as &mut [u8]).unwrap();
    let key = GenericArray::from_slice(&t_key);

    let mut t_nonce = [0u8; 12];
    hex::decode_to_slice(hex::encode(params.resource.nonce.clone()), &mut t_nonce as &mut [u8]).unwrap();
    let nonce = GenericArray::from_slice(&t_nonce);
    
    let t_ciphertext_base = base64::decode(params.resource.ciphertext.clone()).unwrap();
    let cipherdata_length = t_ciphertext_base.len() - auth_key_length;

    let cipherdata = &t_ciphertext_base[0..cipherdata_length];
    let auth_tag = &t_ciphertext_base[cipherdata_length..];

    let mut ciphertext = Vec::from(cipherdata);
    ciphertext.extend_from_slice(&auth_tag);

    let mut t_add = [0u8; 11];  // 这里可能会根据返回值 associated_data 长度而不同，目前应该是固定为 "transaction" 。
    hex::decode_to_slice(hex::encode(params.resource.associated_data.clone()), &mut t_add as &mut [u8]).unwrap();
    let payload = Payload {
        msg: &ciphertext,
        aad: &t_add,
    };
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, payload).unwrap();
    let content = std::str::from_utf8(&plaintext).unwrap();
    let data: WxDecodeData = serde_json::from_str(content).unwrap();

    Ok(data)
}