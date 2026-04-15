// 🔄 Auth 刷新调度器
// 统一管理高亮账号、最常使用账号和 hover 补刷任务。

use std::collections::VecDeque;

/// 刷新优先级层级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefreshTier {
    /// 当前高亮 / 当前 runtime 账号
    Current,
    /// 冷账号仅在 hover 后触发
    HoverOnly,
    /// 最常使用的 top3 预热
    WarmTop3,
}

/// 刷新触发原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshReason {
    TabActivated,
    SelectionChanged,
    ManualRefresh,
}

/// 单个待执行刷新任务。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshTask<K> {
    pub key: K,
    pub tier: RefreshTier,
    pub reason: RefreshReason,
    pub force_refresh: bool,
}

impl<K> RefreshTask<K> {
    pub fn new(key: K, tier: RefreshTier, reason: RefreshReason, force_refresh: bool) -> Self {
        Self {
            key,
            tier,
            reason,
            force_refresh,
        }
    }
}

/// 单飞 + 节流的刷新状态。
#[derive(Debug, Clone)]
pub struct RefreshSchedulerState<K> {
    pending: VecDeque<RefreshTask<K>>,
    in_flight: Option<K>,
    cooldown_ticks: u32,
    interval_ticks: u32,
}

impl<K> RefreshSchedulerState<K>
where
    K: Clone + Eq,
{
    pub fn new(interval_ticks: u32) -> Self {
        Self {
            pending: VecDeque::new(),
            in_flight: None,
            cooldown_ticks: 0,
            interval_ticks,
        }
    }

    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    pub fn push(&mut self, task: RefreshTask<K>) {
        if self.in_flight.as_ref().is_some_and(|key| key == &task.key) {
            return;
        }

        if let Some(position) = self.pending.iter().position(|entry| entry.key == task.key) {
            let mut merged = self
                .pending
                .remove(position)
                .expect("position already checked");
            if task.tier < merged.tier {
                merged.tier = task.tier;
            }
            merged.force_refresh |= task.force_refresh;
            if matches!(task.reason, RefreshReason::ManualRefresh) {
                merged.reason = task.reason;
            }
            self.insert_by_priority(merged);
            return;
        }

        self.insert_by_priority(task);
    }

    pub fn tick(&mut self) {
        if self.cooldown_ticks > 0 {
            self.cooldown_ticks -= 1;
        }
    }

    pub fn set_cooldown(&mut self, ticks: u32) {
        self.cooldown_ticks = ticks;
    }

    pub fn next_ready(&mut self, bypass_cooldown: bool) -> Option<RefreshTask<K>> {
        if self.in_flight.is_some() {
            return None;
        }

        if !bypass_cooldown && self.cooldown_ticks > 0 {
            return None;
        }

        self.pending.pop_front()
    }

    pub fn mark_dispatched(&mut self, task: &RefreshTask<K>) {
        self.in_flight = Some(task.key.clone());
        self.cooldown_ticks = self.interval_ticks;
    }

    pub fn finish(&mut self, key: &K) {
        if self
            .in_flight
            .as_ref()
            .is_some_and(|current| current == key)
        {
            self.in_flight = None;
        }
    }

    #[cfg(test)]
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    #[cfg(test)]
    pub fn cooldown_ticks(&self) -> u32 {
        self.cooldown_ticks
    }

    pub fn in_flight_key(&self) -> Option<&K> {
        self.in_flight.as_ref()
    }

    fn insert_by_priority(&mut self, task: RefreshTask<K>) {
        let position = self
            .pending
            .iter()
            .position(|entry| entry.tier > task.tier)
            .unwrap_or(self.pending.len());
        self.pending.insert(position, task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(
        key: &str,
        tier: RefreshTier,
        reason: RefreshReason,
        force_refresh: bool,
    ) -> RefreshTask<String> {
        RefreshTask::new(key.to_string(), tier, reason, force_refresh)
    }

    #[test]
    fn current_priority_stays_ahead_of_warm_tasks() {
        let mut scheduler = RefreshSchedulerState::new(4);
        scheduler.push(task(
            "warm-1",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        scheduler.push(task(
            "current",
            RefreshTier::Current,
            RefreshReason::TabActivated,
            false,
        ));

        let next = scheduler
            .next_ready(false)
            .expect("current task should be ready after insert");
        assert_eq!(next.key, "current");
        assert_eq!(next.tier, RefreshTier::Current);
    }

    #[test]
    fn duplicate_task_upgrades_priority_without_duplication() {
        let mut scheduler = RefreshSchedulerState::new(4);
        scheduler.push(task(
            "main",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        scheduler.push(task(
            "main",
            RefreshTier::Current,
            RefreshReason::SelectionChanged,
            true,
        ));

        assert_eq!(scheduler.pending_len(), 1);
        let next = scheduler
            .next_ready(false)
            .expect("merged task should remain dispatchable");
        assert_eq!(next.key, "main");
        assert_eq!(next.tier, RefreshTier::Current);
        assert!(next.force_refresh);
    }

    #[test]
    fn hover_task_inserts_before_existing_warm_tasks() {
        let mut scheduler = RefreshSchedulerState::new(4);
        scheduler.push(task(
            "warm-1",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        scheduler.push(task(
            "warm-2",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        scheduler.push(task(
            "hover",
            RefreshTier::HoverOnly,
            RefreshReason::SelectionChanged,
            false,
        ));

        assert_eq!(
            scheduler
                .next_ready(false)
                .expect("hover refresh should dispatch first")
                .key,
            "hover"
        );
        assert_eq!(
            scheduler
                .next_ready(false)
                .expect("warm task should remain after hover task")
                .key,
            "warm-1"
        );
    }

    #[test]
    fn in_flight_blocks_next_dispatch_until_finished() {
        let mut scheduler = RefreshSchedulerState::new(4);
        let current = task(
            "main",
            RefreshTier::Current,
            RefreshReason::TabActivated,
            false,
        );
        scheduler.push(current.clone());
        let dispatched = scheduler
            .next_ready(false)
            .expect("current task should dispatch before entering in-flight state");
        scheduler.mark_dispatched(&dispatched);

        scheduler.push(task(
            "warm-1",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        assert!(scheduler.next_ready(false).is_none());

        scheduler.finish(&"main".to_string());
        scheduler.tick();
        scheduler.tick();
        scheduler.tick();
        scheduler.tick();
        assert_eq!(
            scheduler
                .next_ready(false)
                .expect("warm task should dispatch after in-flight completes")
                .key,
            "warm-1"
        );
    }

    #[test]
    fn cooldown_enforces_one_second_slotting() {
        let mut scheduler = RefreshSchedulerState::new(4);
        let current_task = task(
            "main",
            RefreshTier::Current,
            RefreshReason::TabActivated,
            false,
        );
        scheduler.push(current_task.clone());
        let dispatched = scheduler
            .next_ready(false)
            .expect("current task should dispatch before cooldown starts");
        scheduler.mark_dispatched(&dispatched);
        scheduler.finish(&"main".to_string());

        scheduler.push(task(
            "warm-1",
            RefreshTier::WarmTop3,
            RefreshReason::TabActivated,
            false,
        ));
        assert!(scheduler.next_ready(false).is_none());
        for _ in 0..4 {
            scheduler.tick();
        }
        assert_eq!(scheduler.cooldown_ticks(), 0);
        assert_eq!(
            scheduler
                .next_ready(false)
                .expect("warm task should dispatch after cooldown expires")
                .key,
            "warm-1"
        );
    }
}
