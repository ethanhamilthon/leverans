use std::time::Duration;

pub struct CronManager {
    pub cron_jobs: Vec<String>,
    pub tick: Duration,
}

impl CronManager {
    pub fn new(tick: Duration) -> Self {
        Self {
            cron_jobs: vec![],
            tick,
        }
    }

    pub async fn run(&self) {
        loop {
            tokio::time::sleep(self.tick).await;
        }
    }
}
