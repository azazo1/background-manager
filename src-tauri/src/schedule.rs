//! 调度任务的执行.

use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use tokio::{
    process::{self, Child},
    sync::mpsc,
    task::JoinHandle,
};
use tracing::warn;

use crate::task::{Task, TaskDAO, Trigger};

#[derive(Clone, Debug)]
enum Msg {
    Reconnect(DatabaseConnection),
    // id
    RemoveTask(i64),
    // id
    RunTaskManually(i64),
    // id, enabled
    SwitchTask((i64, bool)),
    SaveTask(Task),
}

#[derive(Clone, Debug)]
enum GuardMsg {
    Reconnect(DatabaseConnection),
    RemoveTask,
    SwitchTask(bool),
    RunTaskManually,
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

impl Scheduler {
    pub(crate) async fn bind(db: DatabaseConnection) -> Self {
        let (tx, rx) = mpsc::channel(100);
        tx.send(Msg::Reconnect(db)).await.ok();
        let schedule_handle = tokio::spawn(async move { Self::schedule(rx).await });
        Scheduler {
            tx,
            schedule_handle,
        }
    }

    async fn schedule(mut rx: mpsc::Receiver<Msg>) -> crate::Result<()> {
        let mut db = DatabaseConnection::default();
        let mut guards: HashMap<i64, mpsc::Sender<GuardMsg>> = HashMap::new();
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
                    if let Some(id) = task.id {
                        if let Err(e) = db.save_task(task.clone()).await {
                            warn!("failed to save task {id}: {e:?}");
                            continue;
                        }
                        if let Some(guard_tx) = guards.get(&id) {
                            guard_tx.send(GuardMsg::RemoveTask).await.ok();
                        }
                        let (guard_tx, guard_rx) = mpsc::channel(10);
                        guards.insert(id, guard_tx);
                        let db = db.clone();
                        tokio::spawn(async move { Self::task_guard(db, task, guard_rx).await });
                    }
                }
                Msg::SwitchTask((id, enabled)) => {
                    if let Some(guard_tx) = guards.get(&id) {
                        guard_tx.send(GuardMsg::SwitchTask(enabled)).await.ok();
                    }
                    if let Err(e) = db.switch_task(id, enabled).await {
                        warn!("failed to switch task {id}: {e:?}");
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

        // 初始化触发器
        // 注意：如果是 Routine，我们需要一个可重置的 sleep
        let mut interval = if let Trigger::Routine(d) = task.trigger {
            Some(tokio::time::interval(d))
        } else {
            None
        };

        loop {
            tokio::select! {
                // 1. 监听外部控制消息
                Some(msg) = rx.recv() => {
                    match msg {
                        GuardMsg::Reconnect(new_conn) => db = new_conn,
                        GuardMsg::RemoveTask => {
                            if let Some(c) = &mut child { c.kill().await.ok(); }
                            break; // 退出 guard, 这里的 exit_code 不需要记录到数据库, 因为数据已经删除了.
                        },
                        GuardMsg::RunTaskManually => {
                            Self::run_and_record(&mut child, &db, &task).await;
                        }
                        GuardMsg::SwitchTask(enabled) => {
                            task.enabled = enabled;
                            if !enabled && let Some(child) = &mut child {
                                child.kill().await.ok();
                            }
                        }
                    }
                }

                // 2. 处理定时触发 (Routine)
                Some(_) = async {
                    if let Some(int) = &mut interval {
                        Some(int.tick().await)
                    } else {
                        None
                    }
                }, if task.enabled && matches!(task.trigger, Trigger::Routine(_)) && child.is_none() => {
                    Self::run_and_record(&mut child, &db, &task).await;
                }

                // 3. 监控进程退出 (KeepAlive 逻辑)
                // 注意：只有当 child 存在时才激活此分支
                status = async {
                    if let Some(c) = &mut child {
                        Some(c.wait().await)
                    } else {
                        None
                    }
                }, if child.is_some() => {
                    if let Some(Ok(exit_status)) = status {
                        let code = exit_status.code().unwrap_or(-1) as i64;
                        db.update_task_exit_code(id, code).await.ok();
                        child = None;

                        // 如果是 KeepAlive，立即重新启动
                        if let Trigger::KeepAlive = task.trigger {
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

    /// 执行任务程序.
    ///
    /// # Note
    ///
    /// 不会操作 database 数据, 需要手动修改.
    async fn run_task(task: Task) -> crate::Result<Child> {
        let mut cmd = process::Command::new(&task.program);
        cmd.args(&task.args);
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
            .send(Msg::SwitchTask((id, enable)))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn save_task(&self, task: Task) -> crate::Result<()> {
        self.tx
            .send(Msg::SaveTask(task))
            .await
            .map_err(failed_to_send)
    }

    pub(crate) async fn remove_task(&self, id: i64) -> crate::Result<()> {
        self.tx
            .send(Msg::RemoveTask(id))
            .await
            .map_err(failed_to_send)
    }
}
