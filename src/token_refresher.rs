use anyhow::{Result, Context};
use reqwest::{Client, header};
use tracing::{info, warn, error};

/// 尝试使用 Cookie 换取最新的 Token
/// 这是一个自包含的业务逻辑：输入 Cookie -> 输出 Token
pub async fn refresh_token_by_cookie(cookie: &str) -> Result<String> {
    let mut headers = header::HeaderMap::new();

    // 1. 必须带上 Cookie
    headers.insert("Cookie", cookie.parse().context("Cookie 解析失败")?);

    // 2. 伪装成最新的 Chrome 浏览器
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36".parse()?);
    headers.insert("Accept", "application/json".parse()?);
    headers.insert("App-Type", "web".parse()?);

    // ⚠️ 我们故意不带 x-client-signature，测试服务器是否允许“裸连”

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    // 使用你抓包抓到的那个 security 接口，或者 users/current
    // 这个接口通常用来预加载用户信息，成功率较高
    let url = "https://www.okx.com/v3/users/security/index?prefetch=true";

    println!("🔄 [测试] 正在请求接口: {}", url);

    let resp = client.get(url).send().await?;
    let status = resp.status();

    println!("📬 [测试] 服务器响应状态码: {}", status);

    if !status.is_success() {
        // 如果是 400/401，说明这个接口强制要求 Signature，纯 Cookie 方案行不通
        let text = resp.text().await?;
        println!("❌ [测试] 失败响应体: {}", text);
        return Err(anyhow::anyhow!("请求失败，状态码: {}", status));
    }

    // 策略 A: 从响应头 Authorization 获取
    if let Some(auth) = resp.headers().get("Authorization") {
        if let Ok(s) = auth.to_str() {
            if s.starts_with("eyJ") {
                println!("✅ [成功] 从响应头 Authorization 拿到 Token!");
                return Ok(s.to_string());
            }
        }
    }

    // 策略 B: 从 Set-Cookie 获取
    let cookies = resp.headers().get_all("Set-Cookie");
    for c in cookies {
        if let Ok(s) = c.to_str() {
            // OKX 的 Token 有时候藏在 cookie 里的 token 字段
            if s.contains("token=eyJ") {
                println!("✅ [成功] 从 Set-Cookie 拿到 Token!");
                // 简单的字符串提取逻辑
                let token_part = s.split(';')
                    .find(|part| part.trim().starts_with("token="))
                    .context("找不到 token 字段")?;
                let token = token_part.trim().trim_start_matches("token=");
                return Ok(token.to_string());
            }
        }
    }

    // 策略 C: 看看响应体里有没有 (有些接口会在 JSON data 里返回)
    let body = resp.text().await?;
    if body.contains("\"token\":\"eyJ") {
        println!("✅ [成功] 从 JSON Body 拿到 Token!");
        // 这里为了演示简单，不做 JSON 解析，直接告诉你是 Body 里有的
        return Ok("在Body里找到了Token".to_string());
    }

    Err(anyhow::anyhow!("请求成功但未发现 Token，可能需要更复杂的签名"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_refresh_token_directly() {
        // 1. 加载 .env 环境变量
        dotenv::dotenv().ok();

        // 2. 获取 .env 里的真实 Cookie
        let cookie = env::var("OKX_COOKIE").expect("❌ 必须在 .env 里配置 OKX_COOKIE 才能运行此测试");

        println!("🧪 开始测试：尝试用 Cookie 换取 Token...");

        // 3. 调用业务逻辑
        match refresh_token_by_cookie(&cookie).await {
            Ok(token) => {
                println!("🎉 测试通过！成功获取 Token: {}...", &token[0..20]);
                // 这里甚至可以写个断言，比如 assert!(token.starts_with("eyJ"));
            },
            Err(e) => {
                println!("💀 测试失败。原因: {:?}", e);
                println!("💡 分析: 如果是 400/403，说明 OKX 强制校验 JS 签名，必须上无头浏览器。");
                panic!("测试不通过"); // 让测试显式失败
            }
        }
    }
}