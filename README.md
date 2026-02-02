```markdown
[🇨🇳 中文版 (Chinese Version)](README_CN.md)
# 📉 USDT Premium Monitor (Rust)


[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-green)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

> **A high-performance, asynchronous CLI tool for real-time monitoring of USDT/USD premium rates across Binance & OKX P2P markets.**

## 📖 Overview

**USDT Premium Monitor** is a specialized arbitrage tool written in Rust. It calculates the price difference (premium) between the P2P USDT price (CNY) and the official USD/CNY forex rate.

When the premium hits a user-defined threshold (e.g., negative premium, meaning USDT is cheaper than USD), the system triggers instant **SMTP email alerts**. It is designed for traders, arbitrageurs, and crypto-enthusiasts who need low-latency data without the bloat of a web browser.

## ✨ Key Features

*   **🚀 High Performance**: Built on `Tokio` for non-blocking, asynchronous I/O.
*   **🌍 Multi-Market Support**:
    *   **Binance P2P**: Fetches lowest sell prices directly from Binance API.
    *   **OKX P2P**: Integrated support for OKX market data.
    *   **Forex Data**: Real-time USD/CNY exchange rates via Yahoo Finance.
*   **🛡️ Robust Network**: Built-in `reqwest` client with **Proxy Support** (HTTP/HTTPS) and auto-retries for unstable network environments.
*   **🔔 Smart Alerts**: Configurable email notifications via SMTP (Gmail, Outlook, etc.) when profit opportunities arise.
*   **📝 Structural Logging**: Thread-safe logging with `tracing` (Console + File rotation) for long-term debugging.

## 🛠️ Project Structure

```text
src/
├── main.rs          # Application entry point & event loop
├── config.rs        # Type-safe configuration (Env vars)
├── logger.rs        # Tracing subscriber & file appender
├── network.rs       # Shared HTTP Client with Proxy logic
├── notifier.rs      # SMTP Email Alert Service
└── sources/         # Data Providers (Strategy Pattern)
    ├── mod.rs       # Trait definitions
    ├── binance.rs   # Binance implementation
    ├── okx.rs       # OKX implementation
    └── forex.rs     # Forex rate fetcher
```

## 🚀 Getting Started

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (Cargo)
*   OpenSSL (Pre-installed on most systems)

### Installation

1.  **Clone the repository**
    ```bash
    git clone https://github.com/oywt/usdt-premium-monitor-rs.git
    cd usdt-premium-monitor-rs
    ```

2.  **Setup Configuration**
    Create a `.env` file in the root directory:
    ```bash
    cp .env.example .env
    ```

3.  **Configure Environment Variables**
    Edit the `.env` file with your settings:

    ```ini
    # --- Threshold Settings ---
    # Trigger alert if (USDT_Price / USD_Rate - 1) <= -0.01 (-1%)
    PREMIUM_THRESHOLD=-0.01
  
    # --- Market Settings ---
    # Minimum trade amount to filter ads (CNY)
    FILTER_AMOUNT=5000
    # Check frequency (Seconds)
    CHECK_INTERVAL=60

    # --- Network ---
    # Optional: Proxy for accessing Binance (e.g., http://127.0.0.1:7890)
    # Leave empty if not needed
    APP_PROXY=

    # --- Email Alerts (SMTP) ---
    SMTP_SERVER=smtp.gmail.com
    SMTP_USER=your_email@gmail.com
    SMTP_PASSWORD=your_app_password
    TARGET_EMAIL=receiver_email@example.com
    ```

### Usage

**Run in Development Mode:**
```bash
cargo run
```

**Build for Production:**
```bash
cargo build --release
./target/release/usdt_premium_monitor
```

## 📊 Example Output

```text
2023-10-27T10:00:00.123Z INFO [main] Starting USDT Premium Monitor...
2023-10-27T10:00:01.456Z INFO [Forex] USD/CNY Rate: 7.315
2023-10-27T10:00:01.789Z INFO [Binance] Best Price: 7.290 (Merchants: 5)
2023-10-27T10:00:01.789Z WARN [Monitor] Negative Premium Detected! -0.34% (USDT: 7.290 < USD: 7.315)
2023-10-27T10:00:02.100Z INFO [Notifier] Alert email sent successfully to user@example.com
```

## 🤝 Contributing

Contributions are welcome!
1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## ⚠️ Disclaimer

This software is for **educational and informational purposes only**. Cryptocurrency trading involves significant risk. The authors are not responsible for any financial losses incurred from the use of this tool. Use it at your own risk.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
```