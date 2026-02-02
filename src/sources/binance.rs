use super::{ExchangeSource};
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
            // ✅ 保存配置里的金额
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
}

#[derive(Deserialize, Debug)]
struct BinAdv {
    price: String,
}

#[async_trait]
impl ExchangeSource for BinanceP2P {
    fn name(&self) -> &str { "Binance" }

    async fn get_buy_price(&self) -> Result<f64> {
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
            return Err(anyhow::anyhow!("Binance 返回了空内容 (可能是被防火墙拦截)"));
        }

        let data: BinResponse = serde_json::from_str(&text)
            .context(format!("解析 Binance JSON 失败，原始内容: {}", text))?;

        if let Some(msg) = data.message {
            if data.code != "000000" && !msg.is_empty() {
                return Err(anyhow::anyhow!("Binance API 错误: {}", msg));
            }
        }

        data.data.first()
            .context("未获取到 Binance 卖单")?
            .adv.price.parse()
            .context("Binance 价格格式错误")
    }
}
