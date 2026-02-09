use crate::config::AppConfig;
use anyhow::Result;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use tracing::{info, error};

pub struct Notifier {
    mailer: SmtpTransport,
    from: String,
    to: String,
}

impl Notifier {
    pub fn new(config: &AppConfig) -> Self {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

        // 构建 SMTP 传输器
        let mailer = SmtpTransport::relay(&config.smtp_server)
            .expect("无效的 SMTP 服务器地址")
            .credentials(creds)
            .build();

        Self {
            mailer,
            from: config.smtp_from.clone(),
            to: config.smtp_to.clone(),
        }
    }

    /// ✅ 新增：专门用于锁单成功的紧急通知
    pub fn send_order_success(&self, source: &str, price: f64, amount_cny: &str) -> Result<()> {
        let subject = format!("🚨🚨🚨 锁单成功 [{} RMB]！请立即付款！", amount_cny);

        let body = format!(
            "⚡ 脚本已成功抢到低价单，请在 15 分钟内付款！\n\n\
            --------------------------------\n\
            交易所: {}\n\
            锁单价格: {:.4}\n\
            锁单金额: {} CNY\n\
            --------------------------------\n\
            ⚠️ 请立即打开 App -> C2C -> 待支付订单 完成转账！\n\
            ⚠️ 超时未付可能导致账号限制！\n\
            \n\
            时间: {}",
            source,
            price,
            amount_cny,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let email = Message::builder()
            .from(self.from.parse()?)
            .to(self.to.parse()?)
            .subject(subject)
            .body(body)?;

        match self.mailer.send(&email) {
            Ok(_) => {
                info!("📧 锁单通知邮件发送成功!");
                Ok(())
            }
            Err(e) => {
                error!("❌ 锁单通知邮件发送失败: {:?}", e);
                Err(e.into())
            }
        }
    }


    /// 发送报警邮件
    /// 为了简化模型，这里使用同步发送 (blocking)，在低频报警场景下是可接受的
    pub fn send_alert(&self, source: &str, usdt_price: f64, forex_rate: f64, premium: f64) -> Result<()> {
        let subject = format!("🚨 负溢价机会 [{}]! 当前溢价: {:.2}%", source, premium * 100.0);
        let body = format!(
            "检测到低溢价/负溢价机会，建议关注！\n\n\
            --------------------------------\n\
            交易所: {}\n\
            USDT 价格: {:.4}\n\
            美元汇率: {:.4}\n\
            实际溢价: {:.4}%\n\
            --------------------------------\n\
            时间: {}",
            source,
            usdt_price,
            forex_rate,
            premium * 100.0,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let email = Message::builder()
            .from(self.from.parse()?)
            .to(self.to.parse()?)
            .subject(subject)
            .body(body)?;

        match self.mailer.send(&email) {
            Ok(_) => {
                info!("📧 邮件发送成功!");
                Ok(())
            }
            Err(e) => {
                error!("❌ 邮件发送失败: {:?}", e);
                Err(e.into())
            }
        }
    }
}