//! 雪花 ID（64-bit）。
//!
//! 布局（与常见 Twitter Snowflake 对齐，便于日后服务端同算法）：
//! - 41 bit：毫秒时间戳（相对自定义纪元）
//! - 10 bit：worker_id（本地客户端预留 0..=15；服务端建议 16..=1023）
//! - 12 bit：序列号
//!
//! JSON / IPC 使用十进制字符串，避免 JS Number 精度问题。

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;

/// 本地客户端固定 worker（规格：0..=15）
pub const LOCAL_WORKER_ID: u16 = 0;

/// 纪元：2024-01-01T00:00:00Z
const EPOCH_MS: i64 = 1_704_067_200_000;

const WORKER_BITS: u64 = 10;
const SEQUENCE_BITS: u64 = 12;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;
const MAX_WORKER: u64 = (1 << WORKER_BITS) - 1;

struct SnowflakeState {
    last_ts: i64,
    sequence: u64,
}

static STATE: Mutex<SnowflakeState> = Mutex::new(SnowflakeState {
    last_ts: 0,
    sequence: 0,
});

/// 当前 Unix 毫秒。时钟早于 1970 时退回 [`EPOCH_MS`]，雪花 ID 与各表时间戳共用此实现。
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(EPOCH_MS)
}

/// 生成下一个雪花 ID（本地 worker = [`LOCAL_WORKER_ID`]）
pub fn next_id() -> Result<i64, AppError> {
    next_id_with_worker(LOCAL_WORKER_ID)
}

pub fn next_id_with_worker(worker_id: u16) -> Result<i64, AppError> {
    let worker = u64::from(worker_id);
    if worker > MAX_WORKER {
        return Err(AppError::Invalid(format!(
            "worker_id out of range: {worker_id}"
        )));
    }

    let mut guard = STATE
        .lock()
        .map_err(|_| AppError::Invalid("snowflake lock poisoned".into()))?;

    let mut ts = now_ms();
    if ts < guard.last_ts {
        return Err(AppError::Invalid("clock moved backwards".into()));
    }

    if ts == guard.last_ts {
        guard.sequence = (guard.sequence + 1) & MAX_SEQUENCE;
        if guard.sequence == 0 {
            while ts <= guard.last_ts {
                ts = now_ms();
            }
        }
    } else {
        guard.sequence = 0;
    }

    guard.last_ts = ts;

    let id = ((ts - EPOCH_MS) as u64) << (WORKER_BITS + SEQUENCE_BITS)
        | (worker << SEQUENCE_BITS)
        | guard.sequence;

    Ok(id as i64)
}

pub fn id_to_string(id: i64) -> String {
    id.to_string()
}

pub fn parse_id(s: &str) -> Result<i64, AppError> {
    s.trim()
        .parse::<i64>()
        .map_err(|_| AppError::Invalid(format!("invalid id: {s}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id_is_monotonic() {
        let a = next_id().expect("a");
        let b = next_id().expect("b");
        assert!(b > a);
    }

    #[test]
    fn roundtrip_string() {
        let id = next_id().expect("id");
        let s = id_to_string(id);
        assert_eq!(parse_id(&s).expect("parse"), id);
        assert!(s.len() >= 10);
    }
}
