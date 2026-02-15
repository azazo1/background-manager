use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(tag = "tag", content = "content")]
pub enum Trigger {
    /// 间隔指定时间触发一次, 如果上一个实例没有结束, 会创建第二个实例.
    Routine(Duration),
    /// 在应用开启时启动.
    Startup,
    /// 保证进程活性, 在子进程退出之后重新启动, 随应用开启时自动启动.
    KeepAlive,
    /// 手动启动.
    #[default]
    Manual,
    /// 指定时间点启动
    Instant(DateTime<FixedOffset>),
}

#[derive(Deserialize, Serialize)]
pub struct Task {
    /// Task id, 不能重复.
    pub id: u64,
    pub name: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub stdin: Option<PathBuf>,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub trigger: Trigger,
    pub enabled: bool,
}
