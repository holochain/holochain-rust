/// when this is dropped - will notify Schedule instance it's ok to proceed
pub struct ScheduleGuard(tokio::sync::mpsc::Sender<()>);

impl Drop for ScheduleGuard {
    fn drop(&mut self) {
        let _ = self.0.try_send(());
    }
}

/// helper for task timing so we don't run things too fast or over each other
pub struct Schedule {
    sender: tokio::sync::mpsc::Sender<()>,
    receiver: tokio::sync::mpsc::Receiver<()>,
    last_true: std::time::Instant,
    interval: std::time::Duration,
}

impl Schedule {
    /// create a new Schedule helper instance
    pub fn new(interval: std::time::Duration) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let last_true = std::time::Instant::now();
        Self {
            sender,
            receiver,
            last_true,
            interval,
        }
    }

    /// returns `true` if we should run another task
    pub fn should_proceed(&mut self) -> bool {
        if self.last_true.elapsed() < self.interval {
            return false;
        }
        if let Ok(()) = self.receiver.try_recv() {
            self.last_true = std::time::Instant::now();
            return true;
        }
        false
    }

    /// get a guard that when dropped will notify us that a task is complete
    pub fn get_guard(&self) -> ScheduleGuard {
        ScheduleGuard(self.sender.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn my_fast_task(_r: ScheduleGuard) {
        tokio::time::delay_for(std::time::Duration::from_millis(10)).await;
    }

    fn spawn_my_fast_task(r: ScheduleGuard) {
        tokio::task::spawn(my_fast_task(r));
    }

    async fn my_slow_task(_r: ScheduleGuard) {
        tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
    }

    fn spawn_my_slow_task(r: ScheduleGuard) {
        tokio::task::spawn(my_slow_task(r));
    }

    async fn check(s: &mut Schedule) {
        loop {
            if s.should_proceed() {
                return;
            }
            tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
        }
    }

    #[tokio::test]
    async fn async_schedule_test() {
        let mut s = Schedule::new(std::time::Duration::from_millis(50));

        let time = std::time::Instant::now();

        spawn_my_slow_task(s.get_guard());

        check(&mut s).await;

        assert!(time.elapsed().as_millis() >= 100);

        let time = std::time::Instant::now();

        spawn_my_fast_task(s.get_guard());
        spawn_my_fast_task(s.get_guard());
        spawn_my_fast_task(s.get_guard());

        check(&mut s).await;

        assert!(time.elapsed().as_millis() >= 50 && time.elapsed().as_millis() < 100);

        spawn_my_slow_task(s.get_guard());

        check(&mut s).await;

        assert!(time.elapsed().as_millis() >= 100);
    }
}
