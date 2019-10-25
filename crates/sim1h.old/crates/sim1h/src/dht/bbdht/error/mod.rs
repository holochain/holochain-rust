#[cfg(ghost)]
use lib3h::error::Lib3hError;
use rusoto_core::RusotoError;
use rusoto_dynamodb::CreateTableError;
use rusoto_dynamodb::DeleteTableError;
use rusoto_dynamodb::DescribeLimitsError;
use rusoto_dynamodb::DescribeTableError;
use rusoto_dynamodb::GetItemError;
use rusoto_dynamodb::ListTablesError;
use rusoto_dynamodb::PutItemError;
use rusoto_dynamodb::ScanError;
use rusoto_dynamodb::UpdateItemError;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub enum BbDhtError {
    // Rusoto mappings
    InternalServerError(String),
    ItemCollectionSizeLimitExceeded(String),
    ProvisionedThroughputExceeded(String),
    ConditionalCheckFailed(String),
    TransactionConflict(String),
    RequestLimitExceeded(String),
    LimitExceeded(String),
    ResourceNotFound(String),
    ResourceInUse(String),
    HttpDispatch(String),
    Credentials(String),
    Validation(String),
    ParseError(String),
    Unknown(String),
    // data handling
    MissingData(String),
    CorruptData(String),
}

impl std::fmt::Display for BbDhtError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (error_type, error_string) = match self {
            BbDhtError::InternalServerError(s) => ("Internal server error", s),
            BbDhtError::ProvisionedThroughputExceeded(s) => ("Provisioned throughput exceeded", s),
            BbDhtError::ItemCollectionSizeLimitExceeded(s) => {
                ("Item collection size limit exceeded", s)
            }
            BbDhtError::ConditionalCheckFailed(s) => ("Conditional check failed", s),
            BbDhtError::TransactionConflict(s) => ("Transaction conflict", s),
            BbDhtError::RequestLimitExceeded(s) => ("Request limit exceeded", s),
            BbDhtError::LimitExceeded(s) => ("Limit exceeded", s),
            BbDhtError::ResourceNotFound(s) => ("Resource not found", s),
            BbDhtError::ResourceInUse(s) => ("Resource in use", s),
            BbDhtError::HttpDispatch(s) => ("HTTP dispatch", s),
            BbDhtError::Credentials(s) => ("Credentials", s),
            BbDhtError::Validation(s) => ("Validation", s),
            BbDhtError::ParseError(s) => ("Parse error", s),
            BbDhtError::Unknown(s) => ("Unknown", s),
            BbDhtError::MissingData(s) => ("Missing data", s),
            BbDhtError::CorruptData(s) => ("Corrupt data", s),
        };
        // Use `self.number` to refer to each positional data point.
        write!(f, "BbDhtError [{}]: {}", error_type, error_string)
    }
}

pub type BbDhtResult<T> = Result<T, BbDhtError>;

impl From<ParseIntError> for BbDhtError {
    fn from(int_error: ParseIntError) -> Self {
        BbDhtError::CorruptData(int_error.to_string())
    }
}

#[cfg(ghost)]
impl From<BbDhtError> for Lib3hError {
    fn from(bb_dht_error: BbDhtError) -> Self {
        Lib3hError::from(bb_dht_error.to_string())
    }
}

impl From<RusotoError<GetItemError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<GetItemError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                GetItemError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                GetItemError::RequestLimitExceeded(err) => BbDhtError::RequestLimitExceeded(err),
                GetItemError::ProvisionedThroughputExceeded(err) => {
                    BbDhtError::ProvisionedThroughputExceeded(err)
                }
                GetItemError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<PutItemError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<PutItemError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                PutItemError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                PutItemError::RequestLimitExceeded(err) => BbDhtError::RequestLimitExceeded(err),
                PutItemError::ProvisionedThroughputExceeded(err) => {
                    BbDhtError::ProvisionedThroughputExceeded(err)
                }
                PutItemError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
                PutItemError::ConditionalCheckFailed(err) => {
                    BbDhtError::ConditionalCheckFailed(err)
                }
                PutItemError::TransactionConflict(err) => BbDhtError::TransactionConflict(err),
                PutItemError::ItemCollectionSizeLimitExceeded(err) => {
                    BbDhtError::ItemCollectionSizeLimitExceeded(err)
                }
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<PutItemError> for BbDhtError {
    fn from(put_item_error: PutItemError) -> BbDhtError {
        BbDhtError::from(RusotoError::Service(put_item_error))
    }
}

impl From<RusotoError<UpdateItemError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<UpdateItemError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                UpdateItemError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                UpdateItemError::RequestLimitExceeded(err) => BbDhtError::RequestLimitExceeded(err),
                UpdateItemError::ProvisionedThroughputExceeded(err) => {
                    BbDhtError::ProvisionedThroughputExceeded(err)
                }
                UpdateItemError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
                UpdateItemError::ConditionalCheckFailed(err) => {
                    BbDhtError::ConditionalCheckFailed(err)
                }
                UpdateItemError::TransactionConflict(err) => BbDhtError::TransactionConflict(err),
                UpdateItemError::ItemCollectionSizeLimitExceeded(err) => {
                    BbDhtError::ItemCollectionSizeLimitExceeded(err)
                }
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<DescribeTableError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<DescribeTableError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                DescribeTableError::InternalServerError(err) => {
                    BbDhtError::InternalServerError(err)
                }
                DescribeTableError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<DescribeLimitsError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<DescribeLimitsError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                DescribeLimitsError::InternalServerError(err) => {
                    BbDhtError::InternalServerError(err)
                }
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<ListTablesError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<ListTablesError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                ListTablesError::InternalServerError(err) => BbDhtError::InternalServerError(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<CreateTableError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<CreateTableError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                CreateTableError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                CreateTableError::LimitExceeded(err) => BbDhtError::LimitExceeded(err),
                CreateTableError::ResourceInUse(err) => BbDhtError::ResourceInUse(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<DeleteTableError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<DeleteTableError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                DeleteTableError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                DeleteTableError::LimitExceeded(err) => BbDhtError::LimitExceeded(err),
                DeleteTableError::ResourceInUse(err) => BbDhtError::ResourceInUse(err),
                DeleteTableError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl From<RusotoError<ScanError>> for BbDhtError {
    fn from(rusoto_error: RusotoError<ScanError>) -> Self {
        match rusoto_error {
            RusotoError::Service(service_error) => match service_error {
                ScanError::InternalServerError(err) => BbDhtError::InternalServerError(err),
                ScanError::RequestLimitExceeded(err) => BbDhtError::RequestLimitExceeded(err),
                ScanError::ProvisionedThroughputExceeded(err) => {
                    BbDhtError::ProvisionedThroughputExceeded(err)
                }
                ScanError::ResourceNotFound(err) => BbDhtError::ResourceNotFound(err),
            },
            RusotoError::HttpDispatch(err) => BbDhtError::HttpDispatch(err.to_string()),
            RusotoError::Credentials(err) => BbDhtError::Credentials(err.to_string()),
            RusotoError::Validation(err) => BbDhtError::Validation(err.to_string()),
            RusotoError::ParseError(err) => BbDhtError::ParseError(err.to_string()),
            RusotoError::Unknown(err) => BbDhtError::Unknown(format!("{:?}", err)),
        }
    }
}

impl std::error::Error for BbDhtError {}
