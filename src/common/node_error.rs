use std::fmt::{Debug, Display, Formatter};

use argon2::password_hash;
use diesel::result::Error as DError;
use lombok::AllArgsConstructor;
use r2d2::Error as R2D2E;
use serde::Serialize;

use crate::utils::uuid_util::generate_uuid;

#[derive(Debug, Clone, Serialize, AllArgsConstructor)]
pub struct NodeError {
    pub uuid: String,
    pub message: String,
}

impl NodeError {
    pub fn hash(step: i8) -> Self {
        NodeError {
            uuid: generate_uuid(),
            message: format!("encryption error (step-{})", step)
                .to_string(),
        }
    }
}

impl Display for NodeError {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        println!(
            "Error(message: {}, uuid: {})",
            self.message, self.uuid
        );
        Ok(())
    }
}


impl From<R2D2E> for NodeError {
    fn from(error: R2D2E) -> Self {
        let uuid = generate_uuid();
        let message = error.to_string();
        Self { uuid, message }
    }
}

impl From<password_hash::Error> for NodeError {
    fn from(error: password_hash::Error) -> Self {
        NodeError {
            uuid: generate_uuid(),
            message: error.to_string(),
        }
    }
}

impl From<DError> for NodeError {
    fn from(error: DError) -> Self {
        let uuid = generate_uuid();
        let message: String;
        match error {
            DError::InvalidCString(e) => {
                message = String::from(
                    format!("InvalidCString: {}", e.to_string())
                );
            }
            DError::DatabaseError(_, info) => {
                message = String::from(info.message());
            }
            DError::NotFound => {
                message = String::from("NotFound");
            }
            DError::QueryBuilderError(e) => {
                message = String::from(e.to_string());
            }
            DError::DeserializationError(e) => {
                message = String::from(e.to_string());
            }
            DError::SerializationError(e) => {
                message = String::from(e.to_string());
            }
            DError::RollbackErrorOnCommit { .. } => {
                message = String::from("RollbackErrorOnCommit")
            }
            DError::RollbackTransaction => {
                message = String::from("RollbackTransaction")
            }
            DError::AlreadyInTransaction => {
                message = String::from("AlreadyInTransaction")
            }
            DError::NotInTransaction => {
                message = String::from("NotInTransaction")
            }
            DError::BrokenTransactionManager => {
                message = String::from("BrokenTransactionManager")
            }
            _ => {
                message = String::from("null")
            }
        };
        Self { uuid, message }
    }
}

