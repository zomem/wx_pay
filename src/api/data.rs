use serde::{Deserialize, Serialize};

// 通用参数
/// 金额，单位 分
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Amount {
    /// 【总金额】 订单总金额，单位为分。
    pub total: u32,
    /// 【货币类型】 CNY：人民币，境内商户号仅支持人民币。
    pub currency: Option<String>,
}
/// 付款用户
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Payer {
    /// 【用户标识】 用户在普通商户AppID下的唯一标识。 下单前需获取到用户的OpenID
    pub openid: String,
}

/// 客户端支付时的 参数信息
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WxData {
    pub sign_type: String,
    pub pay_sign: String,
    pub package: String,
    pub nonce_str: String,
    pub time_stamp: String,
}

/// 商品详情
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GoodsDetail {
    /// 【商户侧商品编码】 由半角的大小写字母、数字、中划线、下划线中的一种或几种组成。
    pub merchant_goods_id: String,
    /// 【微信支付商品编码】 微信支付定义的统一商品编号（没有可不传）
    pub wechatpay_goods_id: Option<String>,
    /// 【商品名称】 商品的实际名称
    pub goods_name: Option<String>,
    /// 【商品数量】 用户购买的数量
    pub quantity: u32,
    /// 【商品单价】 单位为：分。如果商户有优惠，需传输商户优惠后的单价(例如：用户对一笔100元的订单使用了商场发的纸质优惠券100-50，则活动商品的单价应为原单价-50)
    pub unit_price: u32,
}
/// 订单详情
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OrderDetail {
    /// 【订单原价】 1、商户侧一张小票订单可能被分多次支付，订单原价用于记录整张小票的交易金额。
    /// 2、当订单原价与支付金额不相等，则不享受优惠。
    /// 3、该字段主要用于防止同一张小票分多次支付，以享受多次优惠的情况，正常支付订单不必上传此参数。
    pub cost_price: Option<u32>,
    /// 【商品小票ID】 商家小票ID
    pub invoice_id: Option<String>,
    /// 【单品列表】 单品列表信息 条目个数限制：【1，6000】
    pub goods_detail: Vec<GoodsDetail>,
}
/// 门店信息
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StoreInfo {
    /// 【门店编号】 商户侧门店编号
    pub id: String,
    /// 【门店名称】 商户侧门店名称
    pub name: Option<String>,
    /// 【地区编码】 地区编码，详细请见省市区编号对照表。
    pub area_code: Option<String>,
    /// 【详细地址】 详细的商户门店地址
    pub address: Option<String>,
}
/// 支付场景
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommReqSceneInfo {
    /// 【用户终端IP】 用户的客户端IP，支持IPv4和IPv6两种格式的IP地址。
    pub payer_client_ip: String,
    /// 【商户端设备号】 商户端设备号（门店号或收银设备ID）。
    pub device_id: Option<String>,
    /// 【商户门店信息】 商户门店信息
    pub store_info: Option<StoreInfo>,
}
/// 结算信息
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SettleInfo {
    /// 【是否指定分账】 是否指定分账， true：是 false：否
    pub profit_sharing: Option<bool>,
}

/// jsapi 请求参数
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Jsapi {
    /// 【商品描述】 商品描述
    pub description: String,
    /// 【商户订单号】 商户系统内部订单号，只能是数字、大小写字母_-*且在同一个商户号下唯一。
    pub out_trade_no: String,
    /// 【交易结束时间】 订单失效时间，遵循rfc3339标准格式，格式为yyyy-MM-DDTHH:mm:ss+TIMEZONE，yyyy-MM-DD表示年月日，T出现在字符串中，表示time元素的开头，HH:mm:ss表示时分秒，TIMEZONE表示时区（+08:00表示东八区时间，领先UTC8小时，即北京时间）。例如：2015-05-20T13:29:35+08:00表示，北京时间2015年5月20日13点29分35秒。
    pub time_expire: Option<String>,
    /// 【附加数据】 附加数据，在查询API和支付通知中原样返回，可作为自定义参数使用，实际情况下只有支付完成状态才会返回该字段。
    pub attach: Option<String>,
    /// 【订单金额】 订单金额信息
    pub amount: Amount,
    /// 【支付者】 支付者信息。
    pub payer: Payer,
    /// 【优惠功能】 优惠功能
    pub detail: Option<OrderDetail>,
    /// 【场景信息】 支付场景描述
    pub scene_info: Option<CommReqSceneInfo>,
    /// 【结算信息】 结算信息
    pub settle_info: Option<SettleInfo>,
}
