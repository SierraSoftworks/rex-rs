use actix::prelude::*;
use crate::api::APIError;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Owner,
    Contributor,
    Viewer,
    Invalid,
}

impl Default for Role {
    fn default() -> Self {
        Role::Viewer
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "Owner" => Role::Owner,
            "Contributor" => Role::Contributor,
            "Viewer" => Role::Viewer,
            _ => Role::Invalid
        }
    }
}

impl Into<String> for Role {
    fn into(self) -> String {
        match self {
            Role::Owner => "Owner".into(),
            Role::Contributor => "Contributor".into(),
            Role::Viewer => "Viewer".into(),
            Role::Invalid => "INVALID".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub principal_id: u128,
    pub collection_id: u128,
    pub role: Role,
}

#[derive(Debug, Default)]
pub struct StoreRoleAssignment {
    pub collection_id: u128,
    pub principal_id: u128,
    pub role: Role,
}

impl Message for StoreRoleAssignment {
    type Result = Result<RoleAssignment, APIError>;
}

#[derive(Debug, Default)]
pub struct GetRoleAssignment {
    pub collection_id: u128,
    pub principal_id: u128,
}

impl Message for GetRoleAssignment {
    type Result = Result<RoleAssignment, APIError>;
}

#[derive(Debug, Default)]
pub struct GetRoleAssignments {
    pub collection_id: u128,
}

impl Message for GetRoleAssignments {
    type Result = Result<Vec<RoleAssignment>, APIError>;
}

#[derive(Debug, Default)]
pub struct RemoveRoleAssignment {
    pub collection_id: u128,
    pub principal_id: u128,
}

impl Message for RemoveRoleAssignment {
    type Result = Result<(), APIError>;
}
