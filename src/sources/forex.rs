use super::{ForexSource};
use crate::network::build_http_client;
use crate::config::AppConfig;
use async_trait::async_trait;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

pub struct FrankfurterForex {
    client: Client,
}

impl FrankfurterForex {
    pub fn new(config: &AppConfig) -> Result<Self> {
        // 使用带代理的客户端
        Ok(Self { client: build_http_client(config)? })
    }
}

#[derive(Deserialize)]
struct ForexResponse {
    rates: std::collections::HashMap<String, f64>,
}

#[async_trait]
impl ForexSource for FrankfurterForex {
    async fn get_rate(&self) -> Result<f64> {
        let url = "https://api.frankfurter.app/latest?from=USD&to=CNY";

        let resp = self.client.get(url)
            .send().await
            .context("请求 Forex API 失败")?
            .json::<ForexResponse>().await
            .context("解析 Forex JSON 失败")?;

        resp.rates.get("CNY").copied().context("未找到 CNY 汇率数据")
    }
}