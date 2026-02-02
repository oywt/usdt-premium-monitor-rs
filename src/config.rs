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
    pub premium_threshold: f64,

    pub app_proxy: Option<String>,
    pub okx_user_agent: String,
    pub okx_cookie: Option<String>,
    pub filter_amount: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let cwd = env::current_dir().unwrap_or_default();
        // ✅ 使用 info! 记录路径
        info!(path = ?cwd, "📂 开始加载配置...");

        let env_path = cwd.join(".env");
        if env_path.exists() {
            info!("✅ 检测到 .env 文件");
        } else {
            warn!("⚠️ 未检测到 .env 文件，将仅依赖环境变量");
        }

        // 加载 dotenv
        match dotenv::dotenv() {
            Ok(path) => info!(path = ?path, "✅ dotenv 加载成功"),
            Err(e) => warn!(error = ?e, "⚠️ dotenv 加载跳过 (非致命)"),
        }

        let builder = Config::builder()
            .add_source(Environment::default());

        match builder.build()?.try_deserialize() {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                //  error 很重要，写进日志文件，方便排查
                error!(error = ?e, "❌ 配置解析失败");
                Err(e)
            }
        }
    }
}
