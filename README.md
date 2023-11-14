
## 微信支付，rust api

目前版本 apiv3

小程序支付
后台接口，以actix-web为例
```rust
use wx_pay::{decode_wx, Amount, JsapiParams, Payer, WxData, WxPay, WxPayNotify};

#[post("/pay/wx/v3/test")]
pub async fn pay_wx_v3_test() -> Result<impl Responder> {
    let wx_pay = WxPay {
        appid: WECHAT_MINI_APP_ID,
        mchid: WECHAT_PAY_MCH_ID,
        private_key: WECHAT_PRIVATE_KEY,
        serial_no: WECHAT_PAY_SERIAL,
        apiv3_private_key: WECHAT_PAY_APIV3,
        notify_url: WECHAT_PAY_NOTIFY_URL,
        certificates: None,
    };
    let data = wx_pay
        .jsapi(JsapiParams {
            description: "测试122".to_string(),
            out_trade_no: rand_string(16),
            amount: Amount { total: 1 },
            payer: Payer { openid },
        })
        .await
        .unwrap();
    println!("jsapi 返回的 wx_data 为： {:#?}", data);
    return Ok(web::Json(data));
}

/// 微信支付 回调
#[post("/pay/notify_url/action")]
pub async fn pay_notify_url_action(params: web::Json<WxPayNotify>) -> Result<impl Responder> {
    println!("##############  微信支付 回调 #############");
    let t_params = params.0;
    let data = decode_wx(WECHAT_PAY_APIV3, t_params).unwrap();
    println!("##############  微信支付 回调end #############");

    Ok(web::Json(ResultStatus {
        status: 2,
        message: "成功".into(),
    }))
}
```

小程序端：
```js
let res = await post("/pay/wx/v3/test");
Taro.requestPayment({
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
