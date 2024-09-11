use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, KeyInit, Payload},
    Aes256Gcm,
};
use base64::{engine, Engine};
use chrono::Local;
use pkcs8::DecodePrivateKey;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use rsa::{
    sha2::{Digest, Sha256},
    Pkcs1v15Sign, RsaPrivateKey,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{PayReq, Payer};
use crate::WxPay;

/// 获取当前时间戳
pub(crate) fn get_timestamp() -> String {
    let dt = Local::now();
    dt.timestamp().to_string()
}
/// 生成32位随机字符串
pub(crate) fn gen_rand_str() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

/// 签名
pub(crate) fn rsa_sign<T>(private_key: &str, content: T) -> anyhow::Result<String>
where
    T: AsRef<str>,
{
    // 获取私钥对象
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key)?;
    let mut hasher = Sha256::new();
    hasher.update(content.as_ref());
    let hash256 = hasher.finalize();
    let padding = Pkcs1v15Sign::new::<Sha256>();
    let sign_result = private_key.sign(padding, &hash256)?;
    Ok(engine::general_purpose::STANDARD.encode(sign_result))
}

/// 获取请求头
pub(crate) fn get_headers<T>(
    wx_pay: &WxPay,
    pay_req: &PayReq,
    body: &T,
) -> anyhow::Result<HeaderMap>
where
    T: Serialize + DeserializeOwned,
{
    let timestamp = get_timestamp();
    let onece_str = gen_rand_str();
    let method = pay_req.method.as_str();
    let body_string = serde_json::to_string(body)?;

    // 获取签名
    let signature = rsa_sign(
        wx_pay.private_key,
        method.to_string()
            + "\n"
            + pay_req.path.as_str()
            + "\n"
            + timestamp.as_str()
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

    Ok(headers)
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DecodeWxData {
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
    pub amount: DecodeWxDataAmount,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DecodeWxDataAmount {
    pub total: u32,
    pub payer_total: u32,
    pub currency: String,
    pub payer_currency: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WxPayNotifyResource {
    pub algorithm: String,
    pub associated_data: String,
    pub ciphertext: String,
    pub nonce: String,
    pub original_type: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WxPayNotify {
    pub create_time: String,
    pub event_type: String,
    pub id: String,
    pub resource: WxPayNotifyResource,
    pub resource_type: String,
    pub summary: String,
}
/// 微信支付，解密
/// wx_pay_apiv3 为 apiv3 密钥
/// params 为微信回调请求数据
pub fn decode_wx_pay(wx_pay_apiv3: &str, params: WxPayNotify) -> anyhow::Result<DecodeWxData> {
    let auth_key_length = 16;

    let mut t_key = [0u8; 32];
    hex::decode_to_slice(hex::encode(wx_pay_apiv3), &mut t_key as &mut [u8]).unwrap();
    let key = GenericArray::from_slice(&t_key);

    let mut t_nonce = [0u8; 12];
    hex::decode_to_slice(
        hex::encode(params.resource.nonce.clone()),
        &mut t_nonce as &mut [u8],
    )
    .unwrap();
    let nonce = GenericArray::from_slice(&t_nonce);

    let t_ciphertext_base = engine::general_purpose::STANDARD
        .decode(params.resource.ciphertext.clone())
        .unwrap();
    let cipherdata_length = t_ciphertext_base.len() - auth_key_length;

    let cipherdata = &t_ciphertext_base[0..cipherdata_length];
    let auth_tag = &t_ciphertext_base[cipherdata_length..];

    let mut ciphertext = Vec::from(cipherdata);
    ciphertext.extend_from_slice(&auth_tag);

    // 注： AEAD_AES_256_GCM算法的接口细节，请参考rfc5116。微信支付使用的密钥key长度为32个字节，
    // 随机串nonce长度12个字节，associated_data长度小于16个字节并可能为空字符串。
    // 这里可能会根据返回值 associated_data 长度而不同，目前应该是固定为 "transaction" 。
    let t_add = get_slice_arr(params.resource.associated_data);
    let payload = Payload {
        msg: &ciphertext,
        aad: &t_add,
    };
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, payload).unwrap();
    let content = std::str::from_utf8(&plaintext).unwrap();
    let data: DecodeWxData = serde_json::from_str(content).unwrap();

    Ok(data)
}

fn get_slice_arr<'a>(str_data: String) -> Box<[u8]> {
    let arr = match str_data.len() {
        1 => {
            let mut a = [0u8; 1];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        2 => {
            let mut a = [0u8; 2];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        3 => {
            let mut a = [0u8; 3];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        4 => {
            let mut a = [0u8; 4];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        5 => {
            let mut a = [0u8; 5];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        6 => {
            let mut a = [0u8; 6];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        7 => {
            let mut a = [0u8; 7];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        8 => {
            let mut a = [0u8; 8];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        9 => {
            let mut a = [0u8; 9];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        10 => {
            let mut a = [0u8; 10];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        11 => {
            let mut a = [0u8; 11];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        12 => {
            let mut a = [0u8; 12];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        13 => {
            let mut a = [0u8; 13];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        14 => {
            let mut a = [0u8; 14];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        15 => {
            let mut a = [0u8; 15];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        16 => {
            let mut a = [0u8; 16];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        _ => {
            let mut a = [0u8; 0];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
    };
    arr.into_boxed_slice()
}

#[cfg(test)]
mod test {
    use super::rsa_sign;
    use rsa::{
        sha2::{Digest, Sha256},
        Pkcs1v15Sign, RsaPrivateKey,
    };
    #[test]
    fn test_sha2() {
        let mut hasher = Sha256::new();
        hasher.update("niang");
        let hash256 = hasher.finalize();
        println!("ha   {:?}", hash256);
    }
}
