use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

use crate::api::PayReq;
use crate::constants::WX_BASE_URL;
use crate::utils::{get_headers, get_headers_with_serial};
use crate::WxPay;

pub(crate) async fn get<'a, U>(wx_pay: &WxPay<'a>, pay_req: &PayReq) -> anyhow::Result<U>
where
    U: Serialize + DeserializeOwned,
{
    let headers = get_headers(wx_pay, pay_req, None::<&u8>)?;
    let url = WX_BASE_URL.to_string() + &pay_req.path;
    let client = reqwest::Client::new();
    let data: U = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .json()
        .await?;
    Ok(data)
}

pub(crate) async fn post<'a, T, U>(
    wx_pay: &WxPay<'a>,
    pay_req: &PayReq,
    body: &T,
) -> anyhow::Result<U>
where
    T: Serialize + DeserializeOwned,
    U: Serialize + DeserializeOwned,
{
    let headers = get_headers(wx_pay, pay_req, Some(body))?;
    let client = reqwest::Client::new();
    let url = WX_BASE_URL.to_string() + &pay_req.path;

    let response = client.post(url).headers(headers).json(body).send().await?;

    let status = response.status();
    let response_text = response.text().await?;

    // 调试输出响应内容
    println!("Response status: {}", status);
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(anyhow::anyhow!("HTTP error {}: {}", status, response_text));
    }

    let data: U = serde_json::from_str(&response_text).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse response JSON: {}. Response: {}",
            e,
            response_text
        )
    })?;
    Ok(data)
}

/// 支持设置Wechatpay-Serial头的POST请求（用于转账等敏感接口）
pub(crate) async fn post_with_serial<'a, T, U>(
    wx_pay: &WxPay<'a>,
    pay_req: &PayReq,
    body: &T,
    wechatpay_serial: Option<&str>,
) -> anyhow::Result<U>
where
    T: Serialize + DeserializeOwned + Debug,
    U: Serialize + DeserializeOwned,
{
    println!("参数：{:#?}", body);
    let headers = get_headers_with_serial(wx_pay, pay_req, Some(body), wechatpay_serial)?;
    let client = reqwest::Client::new();
    let url = WX_BASE_URL.to_string() + &pay_req.path;

    let response = client.post(url).headers(headers).json(body).send().await?;

    let status = response.status();
    let response_text = response.text().await?;

    // 调试输出响应内容
    println!("Response status: {}", status);
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(anyhow::anyhow!("HTTP error {}: {}", status, response_text));
    }

    let data: U = serde_json::from_str(&response_text).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse response JSON: {}. Response: {}",
            e,
            response_text
        )
    })?;
    Ok(data)
}
