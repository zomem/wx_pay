use serde::{de::DeserializeOwned, Serialize};

use crate::api::PayReq;
use crate::constants::WX_BASE_URL;
use crate::utils::get_headers;
use crate::WxPay;

pub(crate) async fn post<'a, T, U>(
    wx_pay: &WxPay<'a>,
    pay_req: &PayReq,
    body: &T,
) -> anyhow::Result<U>
where
    T: Serialize + DeserializeOwned,
    U: Serialize + DeserializeOwned,
{
    let headers = get_headers(wx_pay, pay_req, body)?;
    let client = reqwest::Client::new();
    let url = WX_BASE_URL.to_string() + &pay_req.path;
    let data: U = client
        .post(url)
        .headers(headers)
        .json(body)
        .send()
        .await?
        .json()
        .await?;
    Ok(data)
}
