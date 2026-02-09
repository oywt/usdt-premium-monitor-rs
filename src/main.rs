mod config;
mod network;
mod notifier;
mod sources;
mod logger;
mod strategy;
mod ordering;
mod token_refresher;
// ✅ 引入下单模块
use crate::strategy::AlertStrategy;
use crate::config::AppConfig;
use crate::sources::{ExchangeSource, ForexSource};
use crate::sources::okx::OkxP2P;
use crate::sources::binance::BinanceP2P;
use crate::sources::forex::FrankfurterForex;
use crate::notifier::Notifier;
use crate::ordering::OrderManager;

use std::time::Duration;
use tokio::time;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 全局日志初始化
    let _guard = logger::init("logs", "info");

    info!("🚀 系统启动中...");

    // 2. 加载配置
    let config = AppConfig::new().expect("❌ 致命错误: 配置加载失败");

    // 打印配置摘要
    info!(
        threshold_alert = %format!("{:.2}%", config.premium_threshold * 100.0),
        threshold_lock = %format!("{:.2}%", config.auto_lock_threshold * 100.0),
        auto_buy = %config.enable_auto_buy,
        "✅ 配置加载完成"
    );

    // 3. 初始化各模块
    let notifier = Notifier::new(&config);
    let forex_provider = FrankfurterForex::new(&config).expect("Forex 模块初始化失败");

    // 初始化下单管理器
    let order_manager = match OrderManager::new(&config) {
        Ok(om) => om,
        Err(e) => {
            error!("❌ 下单模块初始化失败: {:?}", e);
            return Ok(());
        }
    };

    let mut sources: Vec<Box<dyn ExchangeSource>> = vec![];

    match OkxP2P::new(&config) {
        Ok(okx) => sources.push(Box::new(okx)),
        Err(e) => error!(error = ?e, "⚠️ OKX 初始化失败"),
    }

    match BinanceP2P::new(&config) {
        Ok(bin) => sources.push(Box::new(bin)),
        Err(e) => error!(error = ?e, "⚠️ Binance 初始化失败"),
    }

    if sources.is_empty() {
        error!("❌ 所有交易所初始化失败，程序退出");
        return Ok(());
    }

    info!("🚀 监控与狙击服务已就绪...");

    let mut alert_states: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    let mut strategy = AlertStrategy::new(3);
    let mut interval = time::interval(Duration::from_secs(config.check_interval));

    // 5. 主循环
    loop {
        interval.tick().await;

        let forex_rate = match forex_provider.get_rate().await {
            Ok(rate) => rate,
            Err(e) => {
                error!(error = ?e, "❌ 获取 Forex 汇率失败");
                continue;
            }
        };

        for source in &sources {
            process_exchange(
                source.as_ref(),
                forex_rate,
                &config,
                &notifier,
                &order_manager, // ✅ 传入下单管理器
                &mut alert_states,
                &mut strategy,
            ).await;
        }
    }
}

/// 处理单个交易所的逻辑：监控 -> 报警 -> 锁单
async fn process_exchange(
    source: &dyn ExchangeSource,
    forex_rate: f64,
    config: &AppConfig,
    notifier: &Notifier,
    order_manager: &OrderManager,
    alert_states: &mut std::collections::HashMap<String, bool>,
    strategy: &mut AlertStrategy,
) {
    let source_name = source.name();

    // 修改调用：现在返回 Opportunity 结构体
    match source.get_best_opportunity().await {
        Ok(Some(opp)) => {
            let usdt_price = opp.price;
            let premium = (usdt_price - forex_rate) / forex_rate;
            let premium_pct = premium * 100.0;

            info!(
                exchange = %source_name,
                merchant = %opp.merchant_name,
                usdt = usdt_price,
                forex = forex_rate,
                premium = %format!("{:.2}%", premium_pct),
                // 👇 修改了下面这几行
                "📊 市场行情: USDT={:.4} 溢价={:.2}% ( 🎯  期望 {:.2}%) (额度={})",
                usdt_price,
                premium_pct,
                config.auto_lock_threshold * 100.0,
                config.filter_amount
            );

            // --- 核心业务分层逻辑 ---

            // Level 1: 狙击层 (<= -0.24%)
            // 只有开启了自动买入，且是 OKX (目前只实现了 OKX 下单)，且满足阈值
            if config.enable_auto_buy && source_name == "OKX" && premium <= config.auto_lock_threshold {
                warn!(
                    "⚡⚡ [狙击触发] {} 溢价 {:.2}% <= {:.2}%! 正在执行锁单...",
                    source_name, premium_pct, config.auto_lock_threshold * 100.0
                );

                // 执行下单
                // match order_manager.try_place_okx_order(&opp.ad_id, usdt_price, &config.filter_amount).await {
                //     Ok(true) => {
                //         let msg = format!("✅ 自动锁单成功! 价格: {} 溢价: {:.2}%", usdt_price, premium_pct);
                //         // ✅ 替换为新的: 紧急通知
                //         if let Err(e) = notifier.send_order_success(source_name, usdt_price, &config.filter_amount) {
                //             error!("❌ 紧急：锁单成功但邮件发送失败！请手动检查 App！错误: {:?}", e);
                //         }
                //         info!("{}", msg);
                //         // 锁单成功后，休息 10 分钟防止重复下单
                //         tokio::time::sleep(Duration::from_secs(600)).await;
                //         return; // 结束本次循环
                //     },
                //     Err(e) => error!("❌ 下单过程出错: {:?}", e),
                //     Ok(false) => { /* 失败日志已在 order_manager 打印 */ }
                // }
            }

            // Level 2: 报警层 (普通负溢价)
            let is_below_alert_threshold = premium < config.premium_threshold;
            let is_alert_sent = *alert_states.get(source_name).unwrap_or(&false);

            if strategy.should_alert(source_name, is_below_alert_threshold) && !is_alert_sent {
                warn!("🔥 发现负溢价机会 ({:.2}%)! 发送提醒...", premium_pct);
                if notifier.send_alert(source_name, usdt_price, forex_rate, premium).is_ok() {
                    alert_states.insert(source_name.to_string(), true);
                }
            }

            // Level 3: 恢复层
            if is_alert_sent && premium > (config.premium_threshold + 0.005) {
                info!("✅ {} 溢价回归正常，重置报警状态", source_name);
                alert_states.insert(source_name.to_string(), false);
                strategy.reset_if_needed(source_name);
            }
        },
        Ok(None) => {
            warn!("⚠️ {} 未找到满足额度 {} 的有效卖单", source_name, config.filter_amount);
        },
        Err(e) => {
            error!(exchange = %source_name, error = ?e, "⚠️ 获取行情失败");
        }
    }
}