use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Publish,
    SkipTooSoon,
    SkipSameValue,
}

pub struct PublishPolicy {
    min_interval: Duration,
    heartbeat: Duration,
    last_sent: Option<(u64, Instant)>,
}

impl PublishPolicy {
    pub fn new(min_interval: Duration, heartbeat: Duration) -> Self {
        Self {
            min_interval,
            heartbeat,
            last_sent: None,
        }
    }

    pub fn decide(&self, price: u64, now: Instant) -> Decision {
        let Some((last_price, last_at)) = self.last_sent else {
            return Decision::Publish;
        };
        let elapsed = now.duration_since(last_at);
        if elapsed < self.min_interval {
            return Decision::SkipTooSoon;
        }
        if price == last_price && elapsed < self.heartbeat {
            return Decision::SkipSameValue;
        }
        Decision::Publish
    }

    pub fn record_sent(&mut self, price: u64, at: Instant) {
        self.last_sent = Some((price, at));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: Duration = Duration::from_secs(10);
    const HEARTBEAT: Duration = Duration::from_secs(30);

    fn policy_sent(price: u64, at: Instant) -> PublishPolicy {
        let mut policy = PublishPolicy::new(MIN, HEARTBEAT);
        policy.record_sent(price, at);
        policy
    }

    #[test]
    fn first_price_is_always_published() {
        let policy = PublishPolicy::new(MIN, HEARTBEAT);
        assert_eq!(policy.decide(100, Instant::now()), Decision::Publish);
    }

    #[test]
    fn new_price_before_min_interval_is_skipped() {
        let t0 = Instant::now();
        let policy = policy_sent(100, t0);
        assert_eq!(
            policy.decide(101, t0 + Duration::from_secs(5)),
            Decision::SkipTooSoon
        );
    }

    #[test]
    fn new_price_after_min_interval_is_published() {
        let t0 = Instant::now();
        let policy = policy_sent(100, t0);
        assert_eq!(policy.decide(101, t0 + MIN), Decision::Publish);
    }

    #[test]
    fn same_price_before_heartbeat_is_skipped() {
        let t0 = Instant::now();
        let policy = policy_sent(100, t0);
        assert_eq!(
            policy.decide(100, t0 + Duration::from_secs(20)),
            Decision::SkipSameValue
        );
    }

    #[test]
    fn same_price_after_heartbeat_is_published() {
        let t0 = Instant::now();
        let policy = policy_sent(100, t0);
        assert_eq!(policy.decide(100, t0 + HEARTBEAT), Decision::Publish);
    }
}
