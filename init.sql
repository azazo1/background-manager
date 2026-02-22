-- 创建任务主表
CREATE TABLE IF NOT EXISTS tasks (
    -- 使用 BIGINT 匹配 Rust 的 u64
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

    -- 基础信息
    name TEXT NOT NULL,
    program TEXT NOT NULL,

    -- 存储 Vec<String> 的 JSON 字符串，例如: ["--port", "8080"]
    args TEXT NOT NULL DEFAULT '[]',

    -- 可选路径字段
    stdin TEXT,
    stdout TEXT,
    stderr TEXT,

    -- 状态
    enabled BOOLEAN NOT NULL DEFAULT 1,

    -- 触发器逻辑拆分
    -- trigger_tag 存储枚举名: 'Routine', 'Startup', 'KeepAlive', 'Manual', 'Instant', 'UntilSucceed'
    trigger_tag TEXT NOT NULL,

    -- trigger_content 存储对应的数据 JSON
    -- Routine 存: {"secs": 3600, "nanos": 0}
    -- Instant 存: "2026-02-15T23:00:00+08:00"
    -- Startup/Manual/... 存: NULL
    trigger_content TEXT,

    -- 上一次执行状态
    last_exit_code INTEGER,
    last_run_at TEXT
);

-- 为常用过滤字段创建索引，优化查询性能
CREATE INDEX IF NOT EXISTS idx_tasks_enabled ON tasks(enabled);
CREATE INDEX IF NOT EXISTS idx_tasks_trigger_tag ON tasks(trigger_tag);

-- 添加 no_console 列，默认为 0 (false)
ALTER TABLE tasks ADD COLUMN no_console BOOLEAN NOT NULL DEFAULT 0;

-- 添加 working_dir 列, 表示工作目录.
ALTER TABLE tasks ADD COLUMN working_dir TEXT;

-- 添加 env_vars 列, 存储环境变量的 JSON 字符串，例如: {"KEY": "value", "PORT": "8080"}
ALTER TABLE tasks ADD COLUMN env_vars TEXT NOT NULL DEFAULT '{}';