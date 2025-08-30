# 注：当前为测试版

## 微信支付，rust api

**目前版本 apiv3 jsapi**

```rust
let wx_pay = WxPay {
    appid: WECHAT_MINI_APP_ID,
    mchid: WECHAT_PAY_MCH_ID,
    private_key: WECHAT_PRIVATE_KEY,
    serial_no: WECHAT_PAY_SERIAL,
    apiv3_private_key: WECHAT_PAY_APIV3,
    notify_url: WECHAT_PAY_NOTIFY_URL,
    wx_public_key: Some(WECHAT_PUBLIC_KEY), // 可选，用于敏感信息加密
    wx_public_key_id: Some(WECHAT_PUBLIC_KEY_ID), // 可选，微信支付公钥ID
};
```

### jsapi 支付，返回客户端的支付参数信息
```rust
    wx_pay.jsapi
```
### 微信支付订单号查询订单
```rust
    wx_pay.get_transactions_by_id
```
### 商户订单号查询订单
```rust
    wx_pay.get_transactions_by_out_trade_no
```
### 关闭订单
```rust
    wx_pay.close
```
### 退款申请
```rust
    wx_pay.refund
```
### 查寻单笔退款
```rust
    wx_pay.get_refund
```
### 发起转账
```rust
    wx_pay.transfer
```

后台接口，以actix-web为例
```rust
use wx_pay::{TradeState, Transfer, TransferDetail, TransferSceneReportInfo};
use wx_pay::decode::{WxNotify, WxPayResource, decode_wx_notify};
use wx_pay::verification::WxPayVerification;

#[post("/pay/wx/v3/test")]
pub async fn pay_wx_v3_test() -> Result<impl Responder> {
    let wxpay = WxPay {
        appid: WECHAT_MINI_APP_ID,
        mchid: WECHAT_PAY_MCH_ID,
        private_key: WECHAT_PRIVATE_KEY,
        serial_no: WECHAT_PAY_SERIAL,
        apiv3_private_key: WECHAT_PAY_APIV3,
        notify_url: WECHAT_PAY_NOTIFY_URL,
        wx_public_key: Some(WECHAT_PUBLIC_KEY),
        wx_public_key_id: Some(WECHAT_PUBLIC_KEY_ID),
    };
    let data: WxPayData = wxpay
        .jsapi(&Jsapi {
            description: "测试122".to_string(),
            out_trade_no: rand_string(16),
            amount: Amount {
                total: 1,
                ..Default::default()
            },
            payer: Payer { openid },
            ..Default::default()
        })
        .await
        .unwrap();
    return Ok(web::Json(data));
}

/// 发起转账
#[post("/transfer")]
pub async fn transfer_to_user() -> Result<impl Responder> {
    let wxpay = WxPay {
        appid: WECHAT_MINI_APP_ID,
        mchid: WECHAT_PAY_MCH_ID,
        private_key: WECHAT_PRIVATE_KEY,
        serial_no: WECHAT_PAY_SERIAL,
        apiv3_private_key: WECHAT_PAY_APIV3,
        notify_url: WECHAT_PAY_NOTIFY_URL,
        wx_public_key: Some(WECHAT_PUBLIC_KEY), // 用于加密用户姓名
        wx_public_key_id: Some(WECHAT_PUBLIC_KEY_ID), // 微信支付公钥ID
    };

    // 构建转账场景报备信息
    let mut transfer_scene_report_infos = Vec::new();
    transfer_scene_report_infos.push(TransferSceneReportInfo {
        info_type: "活动名称".to_string(),
        info_content: "新会员有礼".to_string(),
    });
    transfer_scene_report_infos.push(TransferSceneReportInfo {
        info_type: "奖励说明".to_string(),
        info_content: "注册会员抽奖一等奖".to_string(),
    });

    let transfer_data = Transfer {
        appid: WECHAT_MINI_APP_ID.to_string(),
        out_bill_no: "T".to_string() + &rand_string(15), // 商户单号
        transfer_scene_id: "1000".to_string(), // 转账场景ID，如现金营销
        openid: user_openid.to_string(),
        user_name: Some("张三".to_string()), // 收款用户姓名，会自动加密
        transfer_amount: 100, // 转账金额，单位分
        transfer_remark: "新会员开通有礼".to_string(),
        notify_url: Some("https://your-domain.com/transfer-notify".to_string()),
        user_recv_perception: Some("现金奖励".to_string()),
        transfer_scene_report_infos,
    };

    let result: TransferDetail = wxpay.transfer(&transfer_data).await.unwrap();
    return Ok(web::Json(result));
}

/// 微信支付 回调
#[post("/pay/notify_url/action")]
pub async fn pay_notify_url_action(body: web::Bytes, req: actix_web::HttpRequest) -> Result<impl Responder> {
    // 1. 用原始 body 进行验签
    let body_str = std::str::from_utf8(&body)?;
    // WECHAT_PAY_PUBKEY 为 微信支付公钥
    let verification = WxPayVerification::new(WECHAT_PAY_PUBKEY.to_string());
    // 获取验签所需的 HTTP 头信息
    let timestamp = req
        .headers()
        .get("Wechatpay-Timestamp")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let nonce = req
        .headers()
        .get("Wechatpay-Nonce")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let signature = req
        .headers()
        .get("Wechatpay-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if WxPayVerification::is_test_signature(signature) {
        return Err(error::ErrorNotAcceptable("测试签名"));
    }
    let is_verifi_ok = verification
        .verify_response(timestamp, nonce, body_str, signature)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    if !is_verifi_ok {
        return Err(error::ErrorNotAcceptable("签名验证失败"));
    }

    // 2. 验签成功后再解析 JSON
    let params: WxNotify = serde_json::from_slice(&body)?;
    if params.event_type != "TRANSACTION.SUCCESS".to_string() {
        // 没返回成功
        return Err(error::ErrorMethodNotAllowed("失败"));
    }
    let data: WxPayResource =
        decode_wx_notify(WECHAT_PAY_APIV3, params).map_err(|e| error::ErrorInternalServerError(e))?;
    if data.trade_state != TradeState::SUCCESS {
        // 没返回成功
        return Err(error::ErrorMethodNotAllowed("失败"));
    }
    println!("回调解密数据： {:#?}", data);

    // ----- 你的业务逻辑 -----

    Ok(web::Json(()))
}
```

公众号/小程序端：
```javascript
let res = await post("/pay/wx/v3/test");
wx.requestPayment({
  timeStamp: res.data.time_stamp,
  nonceStr: res.data.nonce_str,
  package: res.data.package,
  signType: res.data.sign_type,
  paySign: res.data.pay_sign,
  success(res2) {
    console.log("支付返回", res2);
  },
});
```
