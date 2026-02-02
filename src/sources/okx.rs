use super::ExchangeSource;
use crate::config::AppConfig;
use async_trait::async_trait;
use anyhow::{Context, Result};
use reqwest::{Client, Proxy, header};
use serde::Deserialize;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};


pub struct OkxP2P {
    client: Client,
    filter_amount: String,
}

impl OkxP2P {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", config.okx_user_agent.parse()?);
        headers.insert("Accept", "application/json".parse()?);

        if let Some(cookie) = &config.okx_cookie {
            if !cookie.is_empty() {
                headers.insert("Cookie", cookie.parse()?);
            }
        }

        let mut builder = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10));

        if let Some(proxy_url) = &config.app_proxy {
            if !proxy_url.is_empty() {
                builder = builder.proxy(Proxy::all(proxy_url)?);
            }
        }

        Ok(Self {
            client: builder.build()?,
            // ✅ 保存配置里的金额
            filter_amount: config.filter_amount.clone()
        })
    }
}

#[derive(Deserialize, Debug)]
struct OkxResponse {
    code: i64,
    data: OkxData,
}

#[derive(Deserialize, Debug)]
struct OkxData {
    sell: Vec<OkxAd>,
}

#[derive(Deserialize, Debug)]
struct OkxAd {
    price: String,
}

#[async_trait]
impl ExchangeSource for OkxP2P {
    fn name(&self) -> &str { "OKX" }

    async fn get_buy_price(&self) -> Result<f64> {
        // 生成时间戳
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        // ✅ 使用 self.filter_amount 拼接 URL
        let url = format!(
            "https://www.okx.com/v3/c2c/tradingOrders/books?t={}&quoteCurrency=CNY&baseCurrency=USDT&side=sell&paymentMethod=all&userType=all&showTrade=false&sortType=price_asc&quoteMinAmountPerOrder={}",
            timestamp,
            self.filter_amount
        );

        let resp = self.client.get(&url) // 注意这里变成了 &url
            .send().await
            .context("请求 OKX API 失败")?;

        let json: OkxResponse = resp.json().await.context("解析 OKX JSON 失败")?;

        if json.code != 0 {
            return Err(anyhow::anyhow!("OKX API 错误码: {}", json.code));
        }

        if let Some(ad) = json.data.sell.first() {
            ad.price.parse::<f64>().context("OKX 价格格式错误")
        } else {
            Err(anyhow::anyhow!("未获取到 OKX 卖单数据"))
        }
    }
}
