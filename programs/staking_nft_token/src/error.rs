use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Freeze Periode Not Passed")]
    FreezePeriodeNotPassed,

    #[msg("Invalid admin")]
    InvalidAdmin,

    #[msg("Over Flow")]
    Overflow,
}
