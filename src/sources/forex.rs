use super::{ForexSource};
use crate::network::build_http_client;
use crate::config::AppConfig;
use async_trait::async_trait;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{ warn};

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

        // ✅ 优化 1: 增加本地重试逻辑
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            // ✅ 优化 2: 显式关闭持久连接 (Connection: close)
            // 很多免费 API 对长连接支持极差，关掉它能大幅降低此错误
            let resp = self.client.get(url)
                .header("Connection", "close")
                .send()
                .await;

            match resp {
                Ok(r) => {
                    let data = r.json::<ForexResponse>().await?;
                    if let Some(rate) = data.rates.get("CNY") {
                        return Ok(*rate);
                    }
                },
                Err(e) if attempts < max_attempts - 1 => {
                    attempts += 1;
                    warn!(error = %e, attempt = attempts, "⚠️ Forex 请求异常，正在重试...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                },
                Err(e) => return Err(anyhow::anyhow!("Forex API 彻底失败: {}", e)),
            }
        }
        Err(anyhow::anyhow!("未找到 CNY 汇率数据"))
    }
}