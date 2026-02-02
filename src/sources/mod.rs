use async_trait::async_trait;
use anyhow::Result;

pub mod forex;
pub mod okx;
pub mod binance;

/// 交易所数据源接口
#[async_trait]
pub trait ExchangeSource: Send + Sync {
    /// 获取该交易所 USDT 的最低卖单价格 (即用户买入价)
    async fn get_buy_price(&self) -> Result<f64>;

    /// 数据源名称
    fn name(&self) -> &str;
}

/// 法币汇率数据源接口
#[async_trait]
pub trait ForexSource: Send + Sync {
    /// 获取 USD -> CNY 的真实汇率
    async fn get_rate(&self) -> Result<f64>;
}