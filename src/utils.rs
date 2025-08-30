use base64::{engine, Engine};
use chrono::Local;
use pkcs8::{DecodePrivateKey, DecodePublicKey};
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use rsa::{
    pkcs1v15::Pkcs1v15Encrypt,
    rand_core::OsRng,
    sha2::{Digest, Sha256},
    Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::api::PayReq;
use crate::WxPay;

/// 获取当前时间戳
pub(crate) fn get_timestamp() -> i64 {
    let dt = Local::now();
    dt.timestamp()
}
/// 生成32位随机字符串
pub(crate) fn gen_rand_str() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

/// 签名
pub(crate) fn sha_rsa_sign<T>(private_key: &str, content: T) -> anyhow::Result<String>
where
    T: AsRef<str>,
{
    // 获取私钥对象
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key)?;
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(content.as_ref());
    let hash256 = hasher.finalize();
    let padding = Pkcs1v15Sign::new::<Sha256>();
    let sign_result = private_key.sign(padding, &hash256)?;
    Ok(engine::general_purpose::STANDARD.encode(sign_result))
}

/// RSA公钥加密敏感信息
pub(crate) fn rsa_encrypt(public_key_pem: &str, plaintext: &str) -> anyhow::Result<String> {
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)?;
    let mut rng = OsRng;
    let padding = Pkcs1v15Encrypt;
    let encrypted_data = public_key.encrypt(&mut rng, padding, plaintext.as_bytes())?;
    Ok(engine::general_purpose::STANDARD.encode(encrypted_data))
}

/// 获取请求头
pub(crate) fn get_headers<T>(
    wx_pay: &WxPay,
    pay_req: &PayReq,
    body: Option<&T>,
) -> anyhow::Result<HeaderMap>
where
    T: Serialize + DeserializeOwned,
{
    get_headers_with_serial(wx_pay, pay_req, body, None)
}

/// 获取带有Wechatpay-Serial头的请求头（用于转账等需要加密的接口）
pub(crate) fn get_headers_with_serial<T>(
    wx_pay: &WxPay,
    pay_req: &PayReq,
    body: Option<&T>,
    wechatpay_serial: Option<&str>,
) -> anyhow::Result<HeaderMap>
where
    T: Serialize + DeserializeOwned,
{
    let timestamp = get_timestamp();
    let onece_str = gen_rand_str();
    let method = pay_req.method.as_str();
    let body_string = if let Some(b) = body {
        serde_json::to_string(b)?
    } else {
        "".to_string()
    };

    // 获取签名
    let signature = sha_rsa_sign(
        wx_pay.private_key,
        method.to_string()
            + "\n"
            + pay_req.path.as_str()
            + "\n"
            + timestamp.to_string().as_str()
            + "\n"
            + onece_str.as_str()
            + "\n"
            + body_string.as_str()
            + "\n",
    )?;
    // 组装header
    let authorization = "WECHATPAY2-SHA256-RSA2048 mchid=\"".to_string()
        + wx_pay.mchid
        + "\",nonce_str=\""
        + onece_str.as_str()
        + "\",timestamp=\""
        + timestamp.to_string().as_str()
        + "\",signature=\""
        + signature.as_str()
        + "\",serial_no=\""
        + wx_pay.serial_no
        + "\"";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(AUTHORIZATION, authorization.parse().unwrap());
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:28.0) Gecko/20100101 Firefox/28.0"
            .parse()
            .unwrap(),
    );

    // 如果提供了 Wechatpay-Serial，则添加到请求头
    if let Some(serial) = wechatpay_serial {
        headers.insert("Wechatpay-Serial", serial.parse().unwrap());
    }

    Ok(headers)
}

#[cfg(test)]
mod test {
    use super::sha_rsa_sign;
    use rsa::sha2::{Digest, Sha256};
    #[test]
    fn test_sha2() {
        let mut hasher = <Sha256 as Digest>::new();
        hasher.update("niang");
        let hash256 = hasher.finalize();
        println!("ha   {:?}", hash256);
    }

    #[test]
    fn test_rsa_sign() {
        /// 微信支付 v3 密钥
        pub const WECHAT_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC8ZC2Rut7HbZeb
3gOl9uymMJNXT+dFg40P10y8
-----END PRIVATE KEY-----";

        let data = sha_rsa_sign(WECHAT_PRIVATE_KEY, "contentabc4").unwrap();
        println!("rsa签名   {:?}", data);
    }
}
