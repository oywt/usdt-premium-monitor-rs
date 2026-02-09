use super::{ExchangeSource, Opportunity};
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
            filter_amount: config.filter_amount.clone(),
        })
    }
}

// --- JSON 解析结构体 ---
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
    id: String,             // ✅ 关键：获取广告ID
    price: String,
    nickName: String,       // ✅ 获取商家名
    quoteMinAmountPerOrder: String, // ✅ 获取限额
    quoteMaxAmountPerOrder: String,
}

#[async_trait]
impl ExchangeSource for OkxP2P {
    fn name(&self) -> &str { "OKX" }

    async fn get_best_opportunity(&self) -> Result<Option<Opportunity>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let url = format!(
            "https://www.okx.com/v3/c2c/tradingOrders/books?t={}&quoteCurrency=CNY&baseCurrency=USDT&side=sell&paymentMethod=all&userType=all&showTrade=false&sortType=price_asc&quoteMinAmountPerOrder={}",
            timestamp,
            self.filter_amount
        );

        let resp = self.client.get(&url).send().await.context("请求 OKX API 失败")?;
        let json: OkxResponse = resp.json().await.context("解析 OKX JSON 失败")?;

        if json.code != 0 {
            return Err(anyhow::anyhow!("OKX API 错误码: {}", json.code));
        }

        // 找到第一个单子 (默认价格最低)
        if let Some(ad) = json.data.sell.first() {
            let price = ad.price.parse::<f64>().context("OKX 价格格式错误")?;

            return Ok(Some(Opportunity {
                price,
                ad_id: ad.id.clone(),
                merchant_name: ad.nickName.clone(),
                min_amount: ad.quoteMinAmountPerOrder.clone(),
                max_amount: ad.quoteMaxAmountPerOrder.clone(),
            }));
        }

        Ok(None)
    }
}