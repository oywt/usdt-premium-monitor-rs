use crate::config::AppConfig;
use reqwest::{Client, Proxy};
use std::time::Duration;
use anyhow::{Context, Result};
use tracing::info;

pub fn build_http_client(config: &AppConfig) -> Result<Client> {
    let mut builder = Client::builder()
        .user_agent(&config.okx_user_agent)
        .timeout(Duration::from_secs(15))
        .gzip(true)
        .brotli(true)
        .deflate(true);

    if let Some(proxy_url) = &config.app_proxy {
        if !proxy_url.is_empty() {
            let proxy = Proxy::all(proxy_url)
                .context(format!("无效的代理地址: {}", proxy_url))?;
            builder = builder.proxy(proxy);

            info!(proxy = %proxy_url, "🌐 HTTP 代理已启用");
        }
    }

    builder.build().context("HTTP Client 构建失败")
}
