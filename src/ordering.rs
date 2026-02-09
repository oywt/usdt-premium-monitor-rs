use crate::config::AppConfig;
use anyhow::{Result, Context};
use reqwest::{Client, header};
use serde_json::json;
use tracing::{info, error};

pub struct OrderManager {
    client: Client,
    config: AppConfig,
}

impl OrderManager {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", config.okx_user_agent.parse()?);

        // 1. Cookie (保持不变)
        if let Some(cookie) = &config.okx_cookie {
            if !cookie.is_empty() {
                headers.insert("Cookie", cookie.parse()?);
            }
        }

        // 2. ✅ 新增 Authorization (解决 403 Forbidden 的关键)
        if let Some(auth) = &config.okx_authorization {
            if !auth.is_empty() {
                // 直接插入 .env 里的那串 eyJ... 字符
                headers.insert("Authorization", auth.parse()?);
            }
        }

        // 3. 其他 Headers (严格复刻抓包)
        headers.insert("x-c2c-platform", "web".parse()?);
        headers.insert("App-Type", "web".parse()?);
        headers.insert("Origin", "https://www.okx.com".parse()?);
        // headers.insert("Referer", "https://www.okx.com/zh-hans/p2p-markets/cny/buy-usdt".parse()?); // 可选，有些时候不需要
        headers.insert("Content-Type", "application/json".parse()?);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("OrderManager HTTP Client 构建失败")?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// 执行 OKX 锁单
    pub async fn try_place_okx_order(&self, ad_id: &str, price: f64, amount_cny: &str) -> Result<bool> {
        if !self.config.enable_auto_buy {
            return Ok(false);
        }

        // 🟢 修正 1: URL 必须带末尾斜杠 (根据抓包 strict match)
        let url = "https://www.okx.com/v3/c2c/orders/";

        // 辅助计算：抓包里有个 baseAmount (USDT数量)，为了稳妥我们算一下传进去
        let amount_val = amount_cny.parse::<f64>().unwrap_or(0.0);
        let base_amount = if price > 0.0 { amount_val / price } else { 0.0 };
        // 保留 2 位小数 (抓包是 1.42，数字类型)
        let base_amount_val = (base_amount * 100.0).round() / 100.0;

        // 🟢 修正 2: Payload 极简模式 (只传抓包里有的，防止参数校验错误)
        let payload = json!({
            "publicTradingOrderId": ad_id, // 广告 ID
            "totalMoney": amount_cny,      // 法币金额 (字符串)
            "baseAmount": base_amount_val, // USDT 数量 (数字, float)
            "payment": "aliPay",           // 支付方式 (建议用 aliPay 或 wxPay，取决于你的常用方式)
            "sendVerificationUserInfo": 0,
            "key": 1,
            "from": "web",                 // 或者 "WE_Q_1.0.0"
            "fromArea": null
        });

        info!(
            "🔫 [OKX 极简开火] 目标ID: {} | 金额: {} | Payload: {}",
            ad_id, amount_cny, payload.to_string()
        );

        let resp = self.client.post(url)
            .json(&payload)
            .send()
            .await
            .context("发送下单请求失败")?;

        let status = resp.status();
        let text = resp.text().await?;

        info!("📬 服务器响应: [{}] {}", status, text);

        if status.is_success() && text.contains("\"code\":0") {
            info!("✅✅✅ 锁单成功！订单详情: {}", text);
            return Ok(true);
        } else {
            error!("❌ 锁单失败: {}", text);
            return Ok(false);
        }
    }
}