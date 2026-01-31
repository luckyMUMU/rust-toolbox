//! 领域模型 - 值对象

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 序列化辅助函数：从秒数反序列化Duration
pub fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

/// 序列化辅助函数：将Duration序列化为秒数
pub fn serialize_duration_as_secs<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

/// 序列化辅助函数：从秒数反序列化Option<Duration>
pub fn deserialize_option_duration_from_secs<'de, D>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs: Option<u64> = Option::deserialize(deserializer)?;
    Ok(secs.map(Duration::from_secs))
}

/// 序列化辅助函数：将Option<Duration>序列化为秒数
pub fn serialize_option_duration_as_secs<S>(
    duration: &Option<Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(d) => serializer.serialize_some(&d.as_secs()),
        None => serializer.serialize_none(),
    }
}
