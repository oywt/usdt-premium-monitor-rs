```markdown
[🇺🇸 English Version](README.md)
# 📉 USDT Premium Monitor (Rust)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-green)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

> **一个基于 Rust 的高性能命令行工具，用于实时监控 Binance 和 OKX P2P 市场的 USDT 溢价率。**

## 📖 项目简介

**USDT Premium Monitor** 是专为套利者和加密货币交易者设计的监控工具。它能实时计算 USDT 的 P2P 场外价格（CNY）与官方美元汇率（USD/CNY）之间的差价（溢价率）。

当溢价率达到用户设定的阈值（例如出现“负溢价”，即 USDT 价格低于美元汇率）时，系统会立即触发 **SMTP 邮件报警**。得益于 Rust 的高性能特性，该工具极轻量且响应速度远快于浏览器插件。

## ✨ 核心功能

*   **🚀 极致性能**: 基于 `Tokio` 构建的非阻塞异步 I/O，资源占用极低。
*   **🌍 多市场支持**:
    *   **Binance P2P**: 直接对接币安 API 获取最低卖单价格。
    *   **OKX P2P**: 集成欧易 OKX 市场数据支持。
    *   **实时汇率**: 通过 Yahoo Finance 获取实时 USD/CNY 离岸/在岸汇率。
*   **🛡️ 网络稳健**: 内置 `reqwest` 客户端，支持 **HTTP/HTTPS 代理** 配置，并在网络波动时自动重试。
*   **🔔 智能告警**: 支持自定义溢价阈值，触发条件时自动发送邮件通知（支持 Gmail, QQ 邮箱, Outlook 等）。
*   **📝 结构化日志**: 使用 `tracing` 记录日志，支持控制台输出及文件轮转（Log Rotation），便于长期运行和排错。

## 🛠️ 项目结构

```text
src/
├── main.rs          # 程序入口与事件循环
├── config.rs        # 类型安全的配置管理 (环境变量)
├── logger.rs        # 日志系统配置 (Tracing)
├── network.rs       # 共享 HTTP 客户端 (含代理逻辑)
├── notifier.rs      # SMTP 邮件发送服务
└── sources/         # 数据源模块 (策略模式)
    ├── mod.rs       # Trait 接口定义
    ├── binance.rs   # 币安接口实现
    ├── okx.rs       # OKX 接口实现
    └── forex.rs     # 汇率获取实现
```

## 🚀 快速开始

### 前置要求

*   [Rust 工具链 (Cargo)](https://www.rust-lang.org/tools/install)
*   OpenSSL (大多数系统已预装)

### 安装步骤

1.  **克隆仓库**
    ```bash
    git clone https://github.com/oywt/usdt-premium-monitor-rs.git
    cd usdt-premium-monitor-rs
    ```

2.  **创建配置文件**
    在项目根目录创建 `.env` 文件：
    ```bash
    cp .env.example .env
    ```

3.  **配置环境变量**
    编辑 `.env` 文件，填入你的配置信息：

    ```ini
    # --- 阈值设置 ---
    # 报警触发阈值：(USDT价格 / 美元汇率 - 1) <= -0.01
    # -0.01 代表负溢价 1% (即 USDT 比美元便宜 1%)
    PREMIUM_THRESHOLD=-0.01
  
    # --- 市场设置 ---
    # P2P 过滤金额 (CNY)，只看该金额以上的商家广告
    FILTER_AMOUNT=5000
    # 检查频率 (秒)
    CHECK_INTERVAL=60

    # --- 网络设置 ---
    # 代理地址 (访问币安通常需要代理，如 http://127.0.0.1:7890)
    # 如果不需要代理请留空
    APP_PROXY=

    # --- 邮件报警 (SMTP) ---
    SMTP_SERVER=smtp.gmail.com
    SMTP_USER=your_email@gmail.com
    SMTP_PASSWORD=your_app_password
    TARGET_EMAIL=receiver_email@example.com
    ```

### 使用方法

**开发模式运行:**
```bash
cargo run
```

**生产环境编译与运行:**
```bash
cargo build --release
./target/release/usdt_premium_monitor
```

## 📊 运行输出示例

```text
2023-10-27T10:00:00.123Z INFO [main] Starting USDT Premium Monitor...
2023-10-27T10:00:01.456Z INFO [Forex] USD/CNY Rate: 7.315
2023-10-27T10:00:01.789Z INFO [Binance] Best Price: 7.290 (Merchants: 5)
2023-10-27T10:00:01.789Z WARN [Monitor] Negative Premium Detected! -0.34% (USDT: 7.290 < USD: 7.315)
2023-10-27T10:00:02.100Z INFO [Notifier] Alert email sent successfully to user@example.com
```

## 🤝 贡献指南

欢迎提交 Pull Request！
1.  Fork 本项目
2.  创建特性分支 (`git checkout -b feature/AmazingFeature`)
3.  提交改动 (`git commit -m 'Add some AmazingFeature'`)
4.  推送到分支 (`git push origin feature/AmazingFeature`)
5.  提交 Pull Request

## ⚠️ 免责声明

本软件仅供**学习和研究使用**。加密货币交易存在巨大风险，作者不对使用本工具导致的任何财务损失负责。请您自行承担风险。

## 📄 许可证

本项目基于 MIT 许可证开源。详情请参阅 `LICENSE` 文件。
```