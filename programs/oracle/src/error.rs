use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Signer is not authorized: oracle admin or upgrade authority required")]
    Unauthorized,
    #[msg("Price must be greater than zero")]
    InvalidPrice,
    #[msg("New price deviates more than 20% from the previous one")]
    PriceOutOfRange,
    #[msg("Oracle price is stale")]
    StaleOracle,
}
