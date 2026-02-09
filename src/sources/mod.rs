use async_trait::async_trait;
use anyhow::Result;

pub mod forex;
pub mod okx;
pub mod binance;

/// 定义一个结构体，包含下单所需的所有信息
/// 这是业务领域的核心实体：代表一个“可交易的机会”
#[derive(Debug, Clone)]
pub struct Opportunity {
    pub price: f64,
    pub ad_id: String,         // 关键：广告ID，下单必须
    pub merchant_name: String, // 商家名字，打日志用
    pub min_amount: String,    // 最小限额
    pub max_amount: String,    // 最大限额
}

/// 交易所数据源接口
#[async_trait]
pub trait ExchangeSource: Send + Sync {
    /// 获取该交易所 USDT 的最佳机会
    /// 修改返回值：从 f64 变成 Option<Opportunity>
    async fn get_best_opportunity(&self) -> Result<Option<Opportunity>>;

    /// 数据源名称
    fn name(&self) -> &str;
}

/// 法币汇率数据源接口
#[async_trait]
pub trait ForexSource: Send + Sync {
    /// 获取 USD -> CNY 的真实汇率
    async fn get_rate(&self) -> Result<f64>;
}