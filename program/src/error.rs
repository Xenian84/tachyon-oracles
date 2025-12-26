use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Insufficient publishers for quorum")]
    InsufficientPublishers,
    
    #[msg("Publisher is not active")]
    PublisherNotActive,
    
    #[msg("Invalid signature")]
    InvalidSignature,
    
    #[msg("Data is too stale")]
    StaleData,
    
    #[msg("Asset ID mismatch")]
    AssetIdMismatch,
    
    #[msg("Relayer cut basis points exceeds maximum (10000)")]
    InvalidRelayerCut,
    
    #[msg("Unauthorized: admin only")]
    Unauthorized,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Invalid asset ID string")]
    InvalidAssetId,
    
    #[msg("Publish time in future")]
    FuturePublishTime,
    
    #[msg("Duplicate publisher in bundle")]
    DuplicatePublisher,
    
    #[msg("Empty message bundle")]
    EmptyBundle,
}

