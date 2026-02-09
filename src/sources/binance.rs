use super::{ExchangeSource, Opportunity};
use crate::network::build_http_client;
use crate::config::AppConfig;
use async_trait::async_trait;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub struct BinanceP2P {
    client: Client,
    filter_amount: String,
}

impl BinanceP2P {
    pub fn new(config: &AppConfig) -> Result<Self> {
        Ok(Self {
            client: build_http_client(config)?,
            filter_amount: config.filter_amount.clone(),
        })
    }
}

#[derive(Deserialize, Debug)]
struct BinResponse {
    data: Vec<BinAd>,
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize, Debug)]
struct BinAd {
    adv: BinAdv,
    advertiser: BinAdvertiser, // ✅ 新增：获取商家信息
}

#[derive(Deserialize, Debug)]
struct BinAdv {
    price: String,
    advNo: String, // ✅ 获取广告号 (虽然目前不用)
    minSingleTransAmount: String,
    maxSingleTransAmount: String,
}

#[derive(Deserialize, Debug)]
struct BinAdvertiser {
    nickName: String,
}

#[async_trait]
impl ExchangeSource for BinanceP2P {
    fn name(&self) -> &str { "Binance" }

    async fn get_best_opportunity(&self) -> Result<Option<Opportunity>> {
        let url = "https://p2p.binance.com/bapi/c2c/v2/friendly/c2c/adv/search";

        let payload = json!({
            "fiat": "CNY",
            "page": 1,
            "rows": 1,
            "tradeType": "BUY",
            "asset": "USDT",
            "payTypes": [],
            "publisherType": null,
            "transAmount": self.filter_amount
        });

        let resp = self.client.post(url)
            .header("Clienttype", "web")
            .header("Lang", "zh-CN")
            .header("Origin", "https://p2p.binance.com")
            .json(&payload)
            .send()
            .await
            .context("请求 Binance API 失败")?;

        let text = resp.text().await.context("获取 Binance 响应文本失败")?;
        if text.is_empty() {
            return Err(anyhow::anyhow!("Binance 返回空内容"));
        }

        let data: BinResponse = serde_json::from_str(&text)
            .context("解析 Binance JSON 失败")?;

        if let Some(msg) = data.message {
            if data.code != "000000" && !msg.is_empty() {
                return Err(anyhow::anyhow!("Binance API 错误: {}", msg));
            }
        }

        if let Some(ad) = data.data.first() {
            let price = ad.adv.price.parse::<f64>().context("Binance 价格格式错误")?;

            return Ok(Some(Opportunity {
                price,
                ad_id: ad.adv.advNo.clone(),
                merchant_name: ad.advertiser.nickName.clone(),
                min_amount: ad.adv.minSingleTransAmount.clone(),
                max_amount: ad.adv.maxSingleTransAmount.clone(),
            }));
        }

        Ok(None)
    }
}