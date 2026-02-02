// src/logger.rs
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::path::Path;

/// 初始化全局日志系统
/// 返回一个 WorkerGuard，必须在 main 函数中持有它，否则异步日志可能丢失
pub fn init(log_dir: &str, log_level: &str) -> WorkerGuard {
    // 1. 设置文件追加器 (每天滚动一个新文件)
    // 目录: logs/, 文件名: usdt-monitor.YYYY-MM-DD.log
    let file_appender = tracing_appender::rolling::daily(log_dir, "usdt-monitor.log");

    // 2. 包装成非阻塞模式 (避免写文件卡住主线程)
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 3. 定义控制台层 (Console Layer) - 给人看的，带颜色，精简
    let console_layer = fmt::layer()
        .with_target(false) // 控制台不显示模块路径，保持清爽
        .with_thread_ids(false)
        .with_file(false);

    // 4. 定义文件层 (File Layer) - 给机器存档的，详细，无颜色
    let file_layer = fmt::layer()
        .with_ansi(false) // 文件里不要颜色代码
        .with_target(true) // 记录是哪个模块发出的
        .with_thread_names(true)
        .with_writer(non_blocking);

    // 5. 注册全局订阅者
    // RUST_LOG 环境变量优先级最高，其次是传入的 log_level
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(console_layer)
        .with(file_layer)
        .init();

    guard
}
