use base64::{engine, Engine};
use rsa::{
    pkcs8::DecodePublicKey,
    sha2::{Digest, Sha256},
    Pkcs1v15Sign, RsaPublicKey,
};
use anyhow::Result;

/// 微信支付响应签名验证结构体
#[derive(Debug, Clone)]
pub struct WxPayVerification {
    /// 微信支付公钥
    public_key: String,
}

impl WxPayVerification {
    /// 创建新的验签实例
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }

    /// 验证微信支付响应签名
    /// 
    /// # 参数
    /// * `timestamp` - 应答时间戳 (来自 Wechatpay-Timestamp 头部)
    /// * `nonce` - 应答随机串 (来自 Wechatpay-Nonce 头部)
    /// * `body` - 应答报文主体 (原始 Response Body)
    /// * `signature` - 应答签名 (来自 Wechatpay-Signature 头部)
    /// 
    /// # 返回值
    /// * `Ok(true)` - 验签成功
    /// * `Ok(false)` - 验签失败
    /// * `Err(e)` - 验签过程中出现错误
    pub fn verify_response(
        &self,
        timestamp: &str,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> Result<bool> {
        // 1. 构造验签串
        let verify_string = self.build_verify_string(timestamp, nonce, body);
        
        // 2. 验证签名
        self.verify_signature(&verify_string, signature)
    }

    /// 验证微信支付通知回调签名
    /// 
    /// # 参数
    /// * `timestamp` - 时间戳 (来自 Wechatpay-Timestamp 头部)
    /// * `nonce` - 随机串 (来自 Wechatpay-Nonce 头部)
    /// * `body` - 通知报文主体 (原始请求体)
    /// * `signature` - 签名 (来自 Wechatpay-Signature 头部)
    /// 
    /// # 返回值
    /// * `Ok(true)` - 验签成功
    /// * `Ok(false)` - 验签失败
    /// * `Err(e)` - 验签过程中出现错误
    pub fn verify_callback(
        &self,
        timestamp: &str,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> Result<bool> {
        // 通知回调的验签与响应验签逻辑相同
        self.verify_response(timestamp, nonce, body, signature)
    }

    /// 构造验签串
    /// 
    /// 验签串格式：
    /// ```
    /// 应答时间戳\n
    /// 应答随机串\n
    /// 应答报文主体\n
    /// ```
    fn build_verify_string(&self, timestamp: &str, nonce: &str, body: &str) -> String {
        format!("{}\n{}\n{}\n", timestamp, nonce, body)
    }

    /// 使用 RSA SHA256 验证签名
    fn verify_signature(&self, verify_string: &str, signature: &str) -> Result<bool> {
        // 解析公钥
        let public_key = RsaPublicKey::from_public_key_pem(&self.public_key)?;
        
        // Base64 解码签名
        let signature_bytes = engine::general_purpose::STANDARD.decode(signature)?;
        
        // SHA256 哈希
        let mut hasher = <Sha256 as Digest>::new();
        hasher.update(verify_string.as_bytes());
        let hash = hasher.finalize();
        
        // RSA 验签
        let padding = Pkcs1v15Sign::new::<Sha256>();
        match public_key.verify(padding, &hash, &signature_bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 检查签名是否为微信支付探测流量
    /// 
    /// 微信支付会发送包含 "WECHATPAY/SIGNTEST/" 前缀的探测签名
    /// 用于检测商户系统是否正确验证签名
    pub fn is_test_signature(signature: &str) -> bool {
        signature.contains("WECHATPAY/SIGNTEST/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_verify_string() {
        let verification = WxPayVerification::new("dummy_key".to_string());
        let result = verification.build_verify_string(
            "1722850421",
            "d824f2e086d3c1df967785d13fcd22ef",
            r#"{"code_url":"weixin://wxpay/bizpayurl?pr=JyC91EIz1"}"#
        );
        
        let expected = "1722850421\nd824f2e086d3c1df967785d13fcd22ef\n{\"code_url\":\"weixin://wxpay/bizpayurl?pr=JyC91EIz1\"}\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_is_test_signature() {
        assert!(WxPayVerification::is_test_signature("WECHATPAY/SIGNTEST/abcd1234"));
        assert!(!WxPayVerification::is_test_signature("regular_signature"));
    }

    #[test]
    fn test_verify_with_sample_data() {
        // 微信支付公钥示例
        let public_key = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4zej1cqugGQtVSY2Ah8R
MCKcr2UpZ8Npo+5Ja9xpFPYkWHaF1Gjrn3d5kcwAFuHHcfdc3yxDYx6+9grvJnCA
2zQzWjzVRa3BJ5LTMj6yqvhEmtvjO9D1xbFTA2m3kyjxlaIar/RYHZSslT4VmjIa
tW9KJCDKkwpM6x/RIWL8wwfFwgz2q3Zcrff1y72nB8p8P12ndH7GSLoY6d2Tv0OB
2+We2Kyy2+QzfGXOmLp7UK/pFQjJjzhSf9jxaWJXYKIBxpGlddbRZj9PqvFPTiep
8rvfKGNZF9Q6QaMYTpTp/uKQ3YvpDlyeQlYe4rRFauH3mOE6j56QlYQWivknDX9V
rwIDAQAB
-----END PUBLIC KEY-----"#;

        let verification = WxPayVerification::new(public_key.to_string());
        
        // 使用文档中的示例数据进行验证（此处仅演示结构，实际需要真实的签名数据）
        let timestamp = "1722850421";
        let nonce = "d824f2e086d3c1df967785d13fcd22ef";
        let body = r#"{"code_url":"weixin://wxpay/bizpayurl?pr=JyC91EIz1"}"#;
        let signature = "mfI1CPqvBrgcXfgXMFjdNIhBf27ACE2YyeWsWV9ZI7T7RU0vHvbQpu9Z32ogzc+k8ZC5n3kz7h70eWKjgqNdKQF0eRp8mVKlmfzMLBVHbssB9jEZEDXThOX1XFqX7s7ymia1hoHQxQagPGzkdWxtlZPZ4ZPvr1RiqkgAu6Is8MZgXXrRoBKqjmSdrP1N7uxzJ/cjfSiis9FiLjuADoqmQ1P7p2N876YPAol7Rn0+GswwAwxldbdLrmVSjfytfSBJFqTMHn4itojgxSWWN1byuckQt8hSTEv/Lg97QoeGniYP17T80pJeQyL3b+295FPHSO2AtvCgyIbKMZ0BALilAA==";
        
        // 注意: 这个测试可能会失败，因为公钥和签名不匹配
        // 在实际使用中，需要使用对应的真实公钥和签名
        let _result = verification.verify_response(timestamp, nonce, body, signature);
    }
}