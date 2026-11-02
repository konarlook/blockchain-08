use anyhow::Result;
use rand::{rngs::ThreadRng, Rng};

pub trait PriceSource {
    /// Цена SOL в USD с decimals = 6.
    fn next_price(&mut self) -> Result<u64>;
}

const MAX_STEP_BPS: i64 = 50;
const BPS_DENOMINATOR: i128 = 10_000;

pub struct MockPriceSource {
    price: u64,
    rng: ThreadRng,
}

impl MockPriceSource {
    pub fn new(start_price: u64) -> Self {
        Self {
            price: start_price,
            rng: rand::rng(),
        }
    }
}

impl PriceSource for MockPriceSource {
    fn next_price(&mut self) -> Result<u64> {
        let step_bps = self.rng.random_range(-MAX_STEP_BPS..=MAX_STEP_BPS);
        let delta = i128::from(self.price) * i128::from(step_bps) / BPS_DENOMINATOR;
        let next = (i128::from(self.price) + delta).max(1);
        self.price = u64::try_from(next)?;
        Ok(self.price)
    }
}
