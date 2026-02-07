use std::collections::HashMap;

pub struct AlertStrategy {
    // 连续命中计数器
    hit_counters: HashMap<String, u32>,
    // 触发报警所需的连续次数
    min_hits: u32,
}

impl AlertStrategy {
    pub fn new(min_hits: u32) -> Self {
        Self {
            hit_counters: HashMap::new(),
            min_hits,
        }
    }

    /// 核心判断逻辑：输入当前是否满足阈值，返回是否确认报警
    pub fn should_alert(&mut self, source_name: &str, is_below_threshold: bool) -> bool {
        let count = self.hit_counters.entry(source_name.to_string()).or_insert(0);

        if is_below_threshold {
            *count += 1;
            // 只有在正好达到阈值那一刻返回 true，避免持续触发
            *count == self.min_hits
        } else {
            *count = 0; // 价格回归正常，立刻重置计数器
            false
        }
    }

    /// 价格彻底回归后的清理（对应你原有的 0.5% 缓冲区逻辑）
    pub fn reset_if_needed(&mut self, source_name: &str) {
        self.hit_counters.insert(source_name.to_string(), 0);
    }
}