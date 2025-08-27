use serde::{Deserialize, Serialize};

use crate::{
    api::{Amount, Jsapi, PayApi, Payer, WxPayData},
    fetch::{get, post},
    utils::{gen_rand_str, get_timestamp, sha_rsa_sign},
    JsapiParams, Refund, RefundDetail, TransactionDetail,
};

#[derive(Debug)]
pub struct WxPay<'a> {
    /// 【公众号ID】 公众号ID
    pub appid: &'a str,
    /// 【直连商户号】 直连商户号
    pub mchid: &'a str,
    /// 证书key .pem文件
    pub private_key: &'a str,
    pub serial_no: &'a str,
    /// apiv3 私钥，32位字符
    pub api_v3_private_key: &'a str,
    /// 【通知地址】 异步接收微信支付结果通知的回调地址，通知URL必须为外网可访问的URL，不能携带参数。 公网域名必须为HTTPS，如果是走专线接入，使用专线NAT IP或者私有回调域名可使用HTTP
    pub notify_url: &'a str,
}

impl<'a> WxPay<'a> {
    /// jsapi 支付，返回客户端的支付参数信息
    pub async fn jsapi(&self, body: &Jsapi) -> anyhow::Result<WxPayData> {
        let pay_api = PayApi::Jsapi;
        let pay_req = pay_api.get_pay_path(&self);

        let jsapi_params = JsapiParams {
            appid: self.appid.to_string(),
            mchid: self.mchid.to_string(),
            notify_url: self.notify_url.to_string(),
            description: body.description.clone(),
            out_trade_no: body.out_trade_no.clone(),
            amount: body.amount.clone(),
            payer: body.payer.clone(),
            time_expire: body.time_expire.clone(),
            attach: body.attach.clone(),
            goods_tag: body.goods_tag.clone(),
            support_fapiao: body.support_fapiao.clone(),
            detail: body.detail.clone(),
            scene_info: body.scene_info.clone(),
            settle_info: body.settle_info.clone(),
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
        let pay_sign = sha_rsa_sign(
            &self.private_key,
            self.appid.to_string()
                + "\n"
                + now_time.to_string().as_str()
                + "\n"
                + ran_str.as_str()
                + "\n"
                + pack.as_str()
                + "\n",
        )?;
        Ok(WxPayData {
            app_id: Some(self.appid.to_string()),
            sign_type: "RSA".into(),
            pay_sign,
            package: pack,
            nonce_str: ran_str,
            time_stamp: now_time.to_string(),
        })
    }

    /// 微信支付订单号查询订单
    pub async fn get_transactions_by_id(
        &self,
        transaction_id: &str,
    ) -> anyhow::Result<TransactionDetail> {
        let pay_api = PayApi::GetTransactionsById { transaction_id };
        let pay_req = pay_api.get_pay_path(&self);
        let data: TransactionDetail = get(&self, &pay_req).await?;
        Ok(data)
    }

    /// 商户订单号查询订单
    pub async fn get_transactions_by_out_trade_no(
        &self,
        out_trade_no: &str,
    ) -> anyhow::Result<TransactionDetail> {
        let pay_api = PayApi::GetTransactionsByOutTradeNo { out_trade_no };
        let pay_req = pay_api.get_pay_path(&self);
        let data: TransactionDetail = get(&self, &pay_req).await?;
        Ok(data)
    }

    /// 关闭订单，以下情况需要调用关单接口：
    /// 商户订单支付失败需要生成新单号重新发起支付，要对原订单号调用关单，避免重复支付；
    /// 系统下单后，用户支付超时，系统退出不再受理，避免用户继续，请调用关单接口。
    pub async fn close(&self, out_trade_no: &str) -> anyhow::Result<()> {
        let pay_api = PayApi::Close { out_trade_no };
        let pay_req = pay_api.get_pay_path(&self);
        #[derive(Deserialize, Serialize)]
        struct Mchid {
            mchid: String,
        }
        let body = Mchid {
            mchid: self.mchid.to_string(),
        };
        let _ = post(&self, &pay_req, &body).await?;
        Ok(())
    }

    /// 退款申请
    pub async fn refund(&self, body: &Refund) -> anyhow::Result<RefundDetail> {
        let pay_api = PayApi::Refund;
        let pay_req = pay_api.get_pay_path(&self);
        let data: RefundDetail = post(&self, &pay_req, body).await?;
        Ok(data)
    }

    /// 查寻单笔退款
    pub async fn get_refund(&self, out_refund_no: &str) -> anyhow::Result<RefundDetail> {
        let pay_api = PayApi::GetRefund { out_refund_no };
        let pay_req = pay_api.get_pay_path(&self);
        let data: RefundDetail = get(&self, &pay_req).await?;
        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;
    use uuid::Uuid;

    use super::JsapiParams;
    use crate::{Amount, Payer};

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

    #[test]
    fn test_jsapi_params() {
        let a = JsapiParams {
            appid: "wx3dcb".to_string(),
            mchid: "1124".to_string(),
            notify_url: "https:notify".to_string(),
            description: "测试122".to_string(),
            out_trade_no: "190767189563940864".to_string(),
            time_expire: None,
            attach: None,
            goods_tag: None,
            support_fapiao: None,
            amount: Amount {
                total: 1,
                currency: None,
            },
            payer: Payer {
                openid: "oxYrE6123123I".to_string(),
            },
            detail: None,
            scene_info: None,
            settle_info: None,
        };
        let b: serde_json::Value = serde_json::to_value(&a).unwrap();
        println!("bbb  {:?}", b);
    }
}
