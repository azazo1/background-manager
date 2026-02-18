//! 调度任务的执行.

use std::{collections::HashMap, ffi::OsStr, time::Duration};

use sea_orm::DatabaseConnection;
use serde::Serialize;
use tokio::{
    process::{self, Child},
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::Instant,
};
use tracing::warn;

use crate::task::{Task, TaskDAO, Trigger};

#[derive(Debug)]
enum Msg {
    Reconnect(DatabaseConnection),
    // id
    RemoveTask(i64),
    // id
    RunTaskManually(i64),
    // id, enabled
    SwitchTask(i64, bool),
    SaveTask(Box<Task>),
    QueryRunning(i64, oneshot::Sender<TaskStatus>),
    Close,
    // id
    StopTask(i64),
}

#[derive(Debug)]
enum GuardMsg {
    Reconnect(DatabaseConnection),
    RemoveTask,
    SwitchTask(bool),
    RunTaskManually,
    QueryRunning(oneshot::Sender<TaskStatus>),
    Close,
    StopTask,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize)]
pub(crate) enum TaskStatus {
    Suspended,
    Running,
    Idle,
}

/// Suspension 逻辑: 在指定秒数内任务触发失败次数达到指定次数则触发 suspension, 暂停任务的自动执行.
#[derive(Default, Debug)]
struct SuspensionDetector {
    start_time: Option<Instant>,
    failures: usize,
    has_suspended: bool,
}

impl SuspensionDetector {
    const SUSPENSION_TIMESPAN: Duration = Duration::from_secs(3);
    const SUSPENSION_FAILURES: usize = 6;

    #[inline]
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    fn suspended(&self) -> bool {
        self.has_suspended
    }

    /// 记录一次失败, 并返回是否为 suspended 状态.
    fn fail(&mut self) -> bool {
        if self.has_suspended {
            return true;
        }
        self.failures += 1;
        if let Some(start_time) = self.start_time
            && Instant::now().duration_since(start_time) <= Self::SUSPENSION_TIMESPAN
            && self.failures >= Self::SUSPENSION_FAILURES
        {
            self.has_suspended = true;
            return true;
        } else {
            self.start_time = Some(Instant::now());
            if self.failures >= Self::SUSPENSION_FAILURES {
                self.has_suspended = true;
                return true;
            }
        }
        false
    }

    fn reset(&mut self) {
        self.start_time = None;
        self.failures = 0;
        self.has_suspended = false;
    }
}

pub(crate) struct Scheduler {
    tx: mpsc::Sender<Msg>,
    schedule_handle: JoinHandle<crate::Result<()>>,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.schedule_handle.abort();
    }
}

fn failed_to_send(e: mpsc::error::SendError<Msg>) -> crate::Error {
    crate::Error::with_source(
        crate::ErrorKind::Io,
        "failed to send scheduler message",
        Box::new(e),
    )
}

fn failed_to_recv(e: oneshot::error::RecvError) -> crate::Error {
    crate::Error::with_source(
        crate::ErrorKind::Io,
        "failed to receive from guard",
        Box::new(e),
    )
}

impl Scheduler {
    pub(crate) async fn bind(db: DatabaseConnection) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let schedule_handle = tokio::spawn(async move { Self::schedule(rx, db).await });
        Scheduler {
            tx,
            schedule_handle,
        }
    }

    async fn schedule(
        mut rx: mpsc::Receiver<Msg>,
        mut db: DatabaseConnection,
    ) -> crate::Result<()> {
        let mut guards: HashMap<i64, mpsc::Sender<GuardMsg>> = HashMap::new();

        for task in db.list_tasks().await? {
            let Some(id) = task.id else {
                continue;
            };
            let (guard_tx, guard_rx) = mpsc::channel(10);
            guards.insert(id, guard_tx);
            let db = db.clone();
            tokio::spawn(async move { Self::task_guard(db, task, guard_rx).await });
        }

        while let Some(msg) = rx.recv().await {
            match msg {
                Msg::Reconnect(conn) => {
                    db = conn.clone();
                    for guard_tx in guards.values() {
                        guard_tx.send(GuardMsg::Reconnect(conn.clone())).await.ok();
                    }
                }
                Msg::RemoveTask(id) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::RemoveTask).await.ok();
                    };
                    guards.remove(&id);
                    if let Err(e) = db.remove_task(id).await {
                        warn!("failed to remove task {id}: {e:?}");
                    }
                }
                Msg::RunTaskManually(id) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::RunTaskManually).await.ok();
                    };
                }
                Msg::SaveTask(task) => {
                    // 不管是添加还是修改 task, 都删除原来的 guard, 创建新的 guard.
                    let mut task = *task;
                    let id = match db.save_task(task.clone()).await {
                        Ok(id) => id,
                        Err(e) => {
                            if let Some(id) = task.id {
                                warn!("failed to save task {id}: {e:?}");
                            } else {
                                warn!("failed to create task: {e:?}");
                            }
                            continue;
                        }
                    };
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::RemoveTask).await.ok();
                    }
                    task.id = Some(id);
                    let (guard_tx, guard_rx) = mpsc::channel(10);
                    guards.insert(id, guard_tx);
                    let db = db.clone();
                    tokio::spawn(async move { Self::task_guard(db, task, guard_rx).await });
                }
                Msg::SwitchTask(id, enabled) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::SwitchTask(enabled)).await.ok();
                    }
                    if let Err(e) = db.switch_task(id, enabled).await {
                        warn!("failed to switch task {id}: {e:?}");
                    }
                }
                Msg::QueryRunning(id, tx) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::QueryRunning(tx)).await.ok();
                    }
                }
                Msg::Close => {
                    for guard_tx in guards.values() {
                        guard_tx.send(GuardMsg::Close).await.ok();
                    }
                    break;
                }
                Msg::StopTask(id) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::StopTask).await.ok();
                    }
                }
            }
        }
        Ok(())
    }

    async fn task_guard(
        mut db: DatabaseConnection,
        mut task: Task,
        mut rx: mpsc::Receiver<GuardMsg>,
    ) -> crate::Result<()> {
        let id = task.id.unwrap();
        let mut child: Option<Child> = None;
        let mut suspension_detector = SuspensionDetector::new();

        // 初始化触发器
        let mut interval = None;
        let mut instant = None;
        match task.trigger {
            Trigger::Routine(d) => {
                interval = Some(tokio::time::interval(d));
            }
            Trigger::Startup => {
                Self::run_and_record(&mut child, &db, &task).await;
            }
            Trigger::KeepAlive => {
                Self::run_and_record(&mut child, &db, &task).await;
            }
            Trigger::Manual => (),
            Trigger::Instant(date_time) => {
                if task.last_run_at.is_none_or(|l| l < date_time) {
                    let delta = date_time.signed_duration_since(chrono::Local::now());
                    if let Some(target) =
                        Instant::now().checked_add(Duration::from_secs(delta.num_seconds() as u64))
                    {
                        instant = Some(target);
                    }
                }
            }
            Trigger::UntilSucceed => {
                Self::run_and_record(&mut child, &db, &task).await;
            }
        }

        loop {
            tokio::select! {
                // 监听外部控制消息
                Some(msg) = rx.recv() => {
                    match msg {
                        GuardMsg::Reconnect(new_conn) => db = new_conn,
                        GuardMsg::RemoveTask => {
                            if let Some(c) = &mut child {
                                c.kill().await.ok();
                            }
                            break; // 退出 guard, 这里的 exit_code 不需要记录到数据库, 因为数据已经删除了.
                        },
                        GuardMsg::RunTaskManually => {
                            suspension_detector.reset();
                            Self::run_and_record(&mut child, &db, &task).await;
                        }
                        GuardMsg::SwitchTask(enabled) => {
                            suspension_detector.reset();
                            task.enabled = enabled;
                            if !enabled && let Some(child) = &mut child {
                                child.kill().await.ok();
                            }
                        }
                        GuardMsg::QueryRunning(tx) => {
                            tx.send(if child.is_some() {
                                TaskStatus::Running
                            } else if suspension_detector.suspended() {
                                TaskStatus::Suspended
                            } else {
                                TaskStatus::Idle
                            }).ok();
                        }
                        GuardMsg::Close => {
                            if let Some(mut c) = child.take() {
                                c.kill().await.ok();
                                let code = c.wait().await.ok().and_then(|s| s.code()).unwrap_or(-1);
                                db.update_task_exit_code(id, code as i64).await.ok();
                            }
                            break;
                        }
                        GuardMsg::StopTask => {
                            if let Some(mut c) = child.take() {
                                c.kill().await.ok();
                                let code = c.wait().await.ok().and_then(|s| s.code()).unwrap_or(-1);
                                db.update_task_exit_code(id, code as i64).await.ok();
                            }
                        }
                    }
                }

                // 处理定时触发 (Routine)
                Some(_) = async {
                    if let Some(int) = &mut interval {
                        Some(int.tick().await)
                    } else {
                        None
                    }
                } => {
                    Self::run_and_record(&mut child, &db, &task).await;
                }

                // 指定时间触发 (Instant)
                Some(_) = async {
                    if let Some(instant) = &instant {
                        tokio::time::sleep_until(*instant).await;
                        Some(())
                    } else {
                        None
                    }
                } => {
                    Self::run_and_record(&mut child, &db, &task).await;
                }

                // 监控进程退出 (KeepAlive/UntilSucceed 逻辑)
                // 注意：只有当 child 存在时才激活此分支
                status = async {
                    if let Some(c) = &mut child {
                        Some(c.wait().await)
                    } else {
                        None
                    }
                }, if child.is_some() => {
                    if let Some(exit_status) = status {
                        let code = exit_status.ok().and_then(|s| s.code()).unwrap_or(-1) as i64;
                        db.update_task_exit_code(id, code).await.ok();
                        child = None;

                        // 如果是 KeepAlive，立即重新启动
                        if let Trigger::KeepAlive = task.trigger {
                            if code != 0 {
                                suspension_detector.fail();
                            }
                            if !suspension_detector.suspended() {
                                Self::run_and_record(&mut child, &db, &task).await;
                            }
                        } else if let Trigger::UntilSucceed = task.trigger && code != 0 {
                            Self::run_and_record(&mut child, &db, &task).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 辅助函数：运行程序并更新数据库中的最后运行时间, 不会等待子进程结束.
    async fn run_and_record(child: &mut Option<Child>, db: &DatabaseConnection, task: &Task) {
        if child.is_some() {
            return;
        }
        if !task.enabled {
            return;
        }

        let id = task.id.unwrap();
        // 更新最后运行时间
        db.update_task_run_at(id, chrono::Local::now().into())
            .await
            .ok();
        // 启动进程
        match Self::run_task(task.clone()).await {
            Ok(new_child) => *child = Some(new_child),
            Err(e) => warn!("failed to launch task: {e:?}"),
        }
    }

    /// 执行任务程序, 对于macos .app 程序, 使用 open 工具打开, 不支持标准流重定向和获取退出码.
    ///
    /// # Note
    ///
    /// 不会操作 database 数据, 需要手动修改.
    async fn run_task(task: Task) -> crate::Result<Child> {
        let mut cmd = if cfg!(target_os = "macos")
            && task.program.is_dir()
            && matches!(
                task.program.extension().and_then(OsStr::to_str),
                Some("app")
            ) {
            let mut cmd = process::Command::new("/usr/bin/open");
            cmd.arg("-a").arg(&task.program);
            cmd
        } else {
            process::Command::new(&task.program)
        };
        cmd.args(&task.args);
        if let Some(working_dir) = &task.working_dir {
            cmd.current_dir(working_dir);
        } else if let Some(parent) = task.program.parent() {
            cmd.current_dir(parent);
        }
        #[cfg(windows)]
        {
            if task.no_console {
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
        }
        cmd.kill_on_drop(true);

        if let Some(stdin) = &task.stdin
            && let Ok(file) = std::fs::File::open(stdin)
        {
            cmd.stdin(file);
        }
        if let Some(stdout) = &task.stdout
            && let Ok(file) = std::fs::File::create(stdout)
        {
            cmd.stdout(file);
        }

        if let Some(stderr) = &task.stderr
            && let Ok(file) = std::fs::File::create(stderr)
        {
            cmd.stderr(file);
        }

        cmd.spawn().map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                "failed to run task program",
                Box::new(e),
            )
        })
    }

    pub(crate) async fn refresh_connection(&self, db: DatabaseConnection) -> crate::Result<()> {
        self.tx
            .send(Msg::Reconnect(db))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn manually_run_task(&self, id: i64) -> crate::Result<()> {
        self.tx
            .send(Msg::RunTaskManually(id))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn switch_task(&self, id: i64, enable: bool) -> crate::Result<()> {
        self.tx
            .send(Msg::SwitchTask(id, enable))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn save_task(&self, task: Task) -> crate::Result<()> {
        self.tx
            .send(Msg::SaveTask(Box::new(task)))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn remove_task(&self, id: i64) -> crate::Result<()> {
        self.tx
            .send(Msg::RemoveTask(id))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn task_status(&self, id: i64) -> crate::Result<TaskStatus> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Msg::QueryRunning(id, tx))
            .await
            .map_err(failed_to_send)?;
        rx.await.map_err(failed_to_recv)
    }

    /// 终止正在运行的 task.
    pub(crate) async fn stop_task(&self, id: i64) -> crate::Result<()> {
        self.tx
            .send(Msg::StopTask(id))
            .await
            .map_err(failed_to_send)
    }

    /// 关闭所有的 task, 并且关闭后台协程, 后台协程关闭之后其他方法调用将返回 Err.
    pub(crate) async fn close(&self) {
        self.tx.send(Msg::Close).await.ok();
    }
}
