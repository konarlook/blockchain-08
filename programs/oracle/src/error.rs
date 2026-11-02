use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Signer is not oracle admin")]
    Unauthorized,
    #[msg("Price must be grater than zero")]
    InvalidPrice,
    #[msg("New price deviates more than 20% from previous one")]
    PriceOutOfRange,
    #[msg("Oracle price is stale")]
    StaleOracle,
}
