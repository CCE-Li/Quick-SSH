//! 后台监控调度器
//!
//! 为每个目标启动独立采集线程，按固定间隔轮询 [`Collector`]，
//! 结果通过 `mpsc` 通道投递为 [`BackgroundEvent`]，UI 侧在
//! `poll_background_tasks` 中消费，保证 UI 永不因 SSH 阻塞。

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use super::platform::Collector;
use super::snapshot::ServerSnapshot;

/// 后台监控事件（UI 侧统一消费）
#[derive(Debug, Clone)]
pub enum BackgroundEvent {
    /// 单个目标采集完成
    MonitorSnapshot(ServerSnapshot),
}

/// 单个目标的监控任务句柄
struct MonitorTask {
    alias: String,
    /// 停止信号（置为 true 时线程退出）
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// 后台调度器
pub struct MonitorScheduler {
    tasks: Vec<MonitorTask>,
    tx: Sender<BackgroundEvent>,
}

impl MonitorScheduler {
    /// 仅持有发送端、不消费事件的调度器（后续 Phase 拆接其他消费者时使用）
    #[allow(dead_code)]
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::channel();
        Self {
            tasks: Vec::new(),
            tx,
        }
    }

    /// 创建调度器并拿到接收端
    pub fn with_channel() -> (Self, Receiver<BackgroundEvent>) {
        let (tx, rx) = mpsc::channel();
        (
            Self {
                tasks: Vec::new(),
                tx,
            },
            rx,
        )
    }

    /// 添加监控任务（interval 为采集间隔）
    pub fn add_target(&mut self, alias: String, collector: Collector, interval: Duration) {
        let tx = self.tx.clone();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_clone = stop.clone();
        let alias_for_task = alias.clone();
        let alias_for_event = alias.clone();

        // 采集线程独立运行、不保留句柄：停止时仅置位 stop 标志，
        // 线程在本轮采集/等待结束后自行退出（见 stop_target 的说明）。
        std::thread::spawn(move || {
            loop {
                if stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let snapshot = collector.collect(&alias_for_event);
                let _ = tx.send(BackgroundEvent::MonitorSnapshot(snapshot));
                // 支持 0 间隔用于测试（仍让出 CPU）
                let sleep_dur = if interval.is_zero() {
                    Duration::from_millis(10)
                } else {
                    interval
                };
                std::thread::sleep(sleep_dur);
            }
        });

        self.tasks.push(MonitorTask {
            alias: alias_for_task,
            stop,
        });
    }

    /// 停止指定目标的监控
    ///
    /// 仅置位停止标志，**不 join** 采集线程：线程可能正阻塞在一次 SSH 采集或
    /// 采集间隔的等待中，join 会让调用方（UI 主循环）卡顿最长一个采集间隔
    /// （外加一次采集耗时）。线程会在本轮结束后自行退出。
    pub fn stop_target(&mut self, alias: &str) {
        if let Some(pos) = self.tasks.iter().position(|t| t.alias == alias) {
            let task = self.tasks.remove(pos);
            task.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// 停止全部监控任务（同上，只发停止信号，不阻塞等待线程结束）
    pub fn stop_all(&mut self) {
        for task in self.tasks.drain(..) {
            task.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// 当前活跃的监控目标数
    #[allow(dead_code)]
    pub fn active_targets(&self) -> usize {
        self.tasks.len()
    }
}

impl Drop for MonitorScheduler {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::executor::{ExecOutput, RemoteExecutor};

    /// 测试用伪执行器：返回固定输出
    struct FakeExecutor;
    impl RemoteExecutor for FakeExecutor {
        fn exec(&self, _command: &str) -> Result<ExecOutput, crate::monitor::executor::ExecError> {
            Ok(ExecOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
            })
        }
    }

    #[test]
    fn scheduler_emits_snapshot_events() {
        let (mut scheduler, rx) = MonitorScheduler::with_channel();
        let collector = Collector::new(Box::new(FakeExecutor));
        scheduler.add_target("test-host".to_string(), collector, Duration::ZERO);

        // 等待至少一个事件
        let event = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("should receive a snapshot event");
        match event {
            BackgroundEvent::MonitorSnapshot(snap) => {
                assert_eq!(snap.alias, "test-host");
            }
        }

        scheduler.stop_all();
    }

    #[test]
    fn stop_target_removes_task() {
        let (mut scheduler, _rx) = MonitorScheduler::with_channel();
        let collector = Collector::new(Box::new(FakeExecutor));
        scheduler.add_target("a".to_string(), collector, Duration::ZERO);
        assert_eq!(scheduler.active_targets(), 1);
        scheduler.stop_target("a");
        assert_eq!(scheduler.active_targets(), 0);
    }
}
