use serde::Deserialize;
use config::{Config, ConfigError, Environment};
use tracing::{info, warn, error};
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub smtp_to: String,

    pub check_interval: u64,
    pub premium_threshold: f64, // 普通报警阈值 (如 0.0)

    pub app_proxy: Option<String>,
    pub okx_user_agent: String,
    pub okx_cookie: Option<String>,

    // ✅ 新增：Authorization 字段，用于通过 OKX 403 验证
    // 对应 .env 里的 OKX_AUTHORIZATION
    pub okx_authorization: Option<String>,

    pub filter_amount: String,

    // 是否开启自动锁单
    #[serde(default)]
    pub enable_auto_buy: bool,

    // 自动锁单阈值 (默认 -0.24%)
    #[serde(default = "default_lock_threshold")]
    pub auto_lock_threshold: f64,
}

// 默认锁定阈值 -0.24%
fn default_lock_threshold() -> f64 {
    -0.0024
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let cwd = env::current_dir().unwrap_or_default();
        info!(path = ?cwd, "📂 开始加载配置...");

        let env_path = cwd.join(".env");
        if env_path.exists() {
            info!("✅ 检测到 .env 文件");
        } else {
            warn!("⚠️ 未检测到 .env 文件，将仅依赖环境变量");
        }

        match dotenv::dotenv() {
            Ok(_) => info!("✅ dotenv 加载成功"),
            Err(e) => warn!("⚠️ dotenv 加载跳过: {:?}", e),
        }

        let builder = Config::builder()
            .add_source(Environment::default());

        match builder.build()?.try_deserialize() {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                error!(error = ?e, "❌ 配置解析失败");
                Err(e)
            }
        }
    }
}