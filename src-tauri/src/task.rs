use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, FixedOffset};
use sea_orm::{NotSet, Set};
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

#[derive(Deserialize, Serialize, bon::Builder)]
pub struct Task {
    /// Task id, 不能重复, 在数据库中自动递增.
    pub id: Option<i64>,
    #[builder(into)]
    pub name: String,
    #[builder(into)]
    pub program: PathBuf,
    #[builder(into, default)]
    pub args: Vec<String>,
    #[builder(into)]
    pub stdin: Option<PathBuf>,
    #[builder(into)]
    pub stdout: Option<PathBuf>,
    #[builder(into)]
    pub stderr: Option<PathBuf>,
    pub trigger: Trigger,
    #[builder(default = true)]
    pub enabled: bool,
}

impl From<crate::entity::tasks::Model> for Task {
    fn from(m: crate::entity::tasks::Model) -> Self {
        // 解析触发器逻辑
        let trigger = match m.trigger_tag.as_str() {
            "Routine" => m
                .trigger_content
                .and_then(|c| serde_json::from_str(&c).ok())
                .map(Trigger::Routine),
            "Instant" => m
                .trigger_content
                .and_then(|c| serde_json::from_str(&c).ok())
                .map(Trigger::Instant),
            "Startup" => Some(Trigger::Startup),
            "KeepAlive" => Some(Trigger::KeepAlive),
            _ => Some(Trigger::Manual),
        }
        .unwrap_or(Trigger::Manual);

        Task {
            id: Some(m.id),
            name: m.name,
            program: PathBuf::from(m.program),
            // 将 JSON 字符串解析回 Vec<String>
            args: serde_json::from_str(&m.args).unwrap_or_default(),
            stdin: m.stdin.map(PathBuf::from),
            stdout: m.stdout.map(PathBuf::from),
            stderr: m.stderr.map(PathBuf::from),
            trigger,
            enabled: m.enabled,
        }
    }
}

impl From<Task> for crate::entity::tasks::ActiveModel {
    fn from(t: Task) -> Self {
        // 拆分 Trigger 为 tag 和 content
        let (tag, content) = match t.trigger {
            Trigger::Routine(d) => ("Routine", Some(serde_json::to_string(&d).unwrap())),
            Trigger::Instant(i) => ("Instant", Some(serde_json::to_string(&i).unwrap())),
            Trigger::Startup => ("Startup", None),
            Trigger::KeepAlive => ("KeepAlive", None),
            Trigger::Manual => ("Manual", None),
        };

        Self {
            id: match t.id {
                Some(id) => Set(id),
                None => NotSet, // 数据库会自动递增生成 ID
            },
            name: Set(t.name),
            program: Set(t.program.to_string_lossy().into_owned()),
            // 将 Vec<String> 序列化为 JSON 字符串
            args: Set(serde_json::to_string(&t.args).unwrap_or_else(|_| "[]".to_string())),
            stdin: Set(t.stdin.map(|p| p.to_string_lossy().into_owned())),
            stdout: Set(t.stdout.map(|p| p.to_string_lossy().into_owned())),
            stderr: Set(t.stderr.map(|p| p.to_string_lossy().into_owned())),
            enabled: Set(t.enabled),
            trigger_tag: Set(tag.to_string()),
            trigger_content: Set(content),
        }
    }
}
