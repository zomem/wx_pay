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

后台接口，以actix-web为例
```rust
use wx_pay::{decode_wx_pay, Amount, Jsapi, Payer, WxPayData, WxPay, WxPayNotify};

#[post("/pay/wx/v3/test")]
pub async fn pay_wx_v3_test() -> Result<impl Responder> {
    let wxpay = WxPay {
        appid: WECHAT_MINI_APP_ID,
        mchid: WECHAT_PAY_MCH_ID,
        private_key: WECHAT_PRIVATE_KEY,
        serial_no: WECHAT_PAY_SERIAL,
        apiv3_private_key: WECHAT_PAY_APIV3,
        notify_url: WECHAT_PAY_NOTIFY_URL,
    };
    let data: WxPayData = wxpay
        .jsapi(&Jsapi {
            description: "测试122".to_string(),
            out_trade_no: rand_string(16),
            amount: Amount { total: 1 },
            payer: Payer { openid },
        })
        .await
        .unwrap();
    return Ok(web::Json(data));
}

/// 微信支付 回调
#[post("/pay/notify_url/action")]
pub async fn pay_notify_url_action(params: web::Json<WxPayNotify>) -> Result<impl Responder> {
    let params = params.0;
    let data = decode_wx_pay(WECHAT_PAY_APIV3, params).unwrap();
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
