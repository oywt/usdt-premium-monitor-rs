mod config;
mod network;
mod notifier;
mod sources;
mod logger;

use crate::config::AppConfig;
use crate::sources::{ExchangeSource, ForexSource};
use crate::sources::okx::OkxP2P;
use crate::sources::binance::BinanceP2P;
use crate::sources::forex::FrankfurterForex;
use crate::notifier::Notifier;
use std::time::Duration;
use tokio::time;
use tracing::{info, error, warn, debug, instrument};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 全局日志初始化
    // _guard 必须持有到 main 结束，否则异步日志（文件写入）会丢失
    let _guard = logger::init("logs", "info");

    info!("🚀 系统启动中...");

    // 2. 加载配置
    let config = AppConfig::new().expect("❌ 致命错误: 配置加载失败，请检查 logs");

    // 使用结构化日志打印摘要
    info!(
        proxy = ?config.app_proxy,
        threshold = %format!("{:.2}%", config.premium_threshold * 100.0),
        interval = %config.check_interval,
        filter_amount = %config.filter_amount,
        "✅ 配置加载完成"
    );

    // 3. 初始化各模块
    let notifier = Notifier::new(&config);
    let forex_provider = FrankfurterForex::new(&config).expect("Forex 模块初始化失败");

    let mut sources: Vec<Box<dyn ExchangeSource>> = vec![];

    // 初始化 OKX
    match OkxP2P::new(&config) {
        Ok(okx) => sources.push(Box::new(okx)),
        Err(e) => error!(error = ?e, "⚠️ OKX 初始化失败"),
    }

    // 初始化 Binance
    match BinanceP2P::new(&config) {
        Ok(bin) => sources.push(Box::new(bin)),
        Err(e) => error!(error = ?e, "⚠️ Binance 初始化失败"),
    }

    if sources.is_empty() {
        error!("❌ 所有交易所初始化失败，程序退出");
        return Ok(());
    }

    info!("🚀 监控服务已就绪，开始循环...");

    let mut alert_states: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    let mut interval = time::interval(Duration::from_secs(config.check_interval));

    // 5. 主循环
    loop {
        interval.tick().await;

        // 5.1 获取基准汇率
        let forex_rate = match forex_provider.get_rate().await {
            Ok(rate) => rate,
            Err(e) => {
                error!(error = ?e, "❌ 获取 Forex 汇率失败");
                continue;
            }
        };

        // 5.2 遍历所有交易所
        for source in &sources {
            // 将处理逻辑独立，保持 main 清爽
            process_exchange(
                source.as_ref(),
                forex_rate,
                &config,
                &notifier,
                &mut alert_states
            ).await;
        }
    }
}


async fn process_exchange(
    source: &dyn ExchangeSource,
    forex_rate: f64,
    config: &AppConfig,
    notifier: &Notifier,
    alert_states: &mut std::collections::HashMap<String, bool>
) {
    let source_name = source.name();

    match source.get_buy_price().await {
        Ok(usdt_price) => {
            let premium = (usdt_price - forex_rate) / forex_rate;
            let premium_pct = premium * 100.0;



            info!(
                exchange = %source_name,
                usdt = usdt_price,
                forex = forex_rate,
                premium = premium_pct,
                "📊 市场行情: USDT={:.4} 溢价={:.2}%",
                usdt_price,
                premium_pct
            );

            let is_alert_sent = *alert_states.get(source_name).unwrap_or(&false);

            if premium < config.premium_threshold {
                if !is_alert_sent {
                    warn!(
                        exchange = %source_name,
                        premium = premium_pct,
                        "🔥 发现负溢价机会! 当前溢价: {:.2}%",
                        premium_pct
                    );

                    match notifier.send_alert(source_name, usdt_price, forex_rate, premium) {
                        Ok(_) => {
                            alert_states.insert(source_name.to_string(), true);
                        },
                        Err(e) => error!(exchange = %source_name, error = ?e, "❌ 邮件发送失败"),
                    }
                }
            } else {
                // 缓冲区重置 (Threshold + 0.5%)
                if is_alert_sent && premium > (config.premium_threshold + 0.005) {
                    info!(exchange = %source_name, "✅ 溢价回归正常，重置报警状态");
                    alert_states.insert(source_name.to_string(), false);
                }
            }
        },
        Err(e) => {
            error!(exchange = %source_name, error = ?e, "⚠️ 获取价格失败");
        }
    }
}

