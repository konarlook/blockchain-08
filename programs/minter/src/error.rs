use anchor_lang::prelude::*;

#[error_code(offset = 7000)]
pub enum MinterError {
    #[msg("Oracle decimals do not match the expected format")]
    InvalidOracleDecimals,
    #[msg("Oracle price must be greater than zero")]
    InvalidOraclePrice,
    #[msg("Arithmetic overflow while calculating the fee")]
    MathOverflow,
    #[msg("Treasury account does not match the config")]
    InvalidTreasury,
    #[msg("Token decimals are out of the allowed range")]
    InvalidTokenDecimals,
    #[msg("Initial supply must be greater than zero")]
    InvalidSupply,
}
