use rex_api::RoleAssignmentV3;

use super::{Id, format_id, parse_id};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Role {
    Owner,
    Contributor,
    #[default]
    Viewer,
    Invalid,
}

impl Role {
    /// Whether this role may add, change, or remove ideas in the collection.
    pub fn can_write(&self) -> bool {
        matches!(self, Role::Owner | Role::Contributor)
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "Owner" => Role::Owner,
            "Contributor" => Role::Contributor,
            "Viewer" => Role::Viewer,
            _ => Role::Invalid,
        }
    }
}

impl From<Role> for String {
    fn from(val: Role) -> Self {
        match val {
            Role::Owner => "Owner".into(),
            Role::Contributor => "Contributor".into(),
            Role::Viewer => "Viewer".into(),
            Role::Invalid => "INVALID".into(),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from(*self))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoleAssignment {
    pub user_id: Id,
    pub collection_id: Id,
    pub role: Role,
}

impl From<RoleAssignment> for RoleAssignmentV3 {
    fn from(assignment: RoleAssignment) -> Self {
        Self {
            user_id: Some(format_id(assignment.user_id)),
            collection_id: Some(format_id(assignment.collection_id)),
            role: assignment.role.into(),
        }
    }
}

impl From<RoleAssignmentV3> for RoleAssignment {
    fn from(val: RoleAssignmentV3) -> Self {
        RoleAssignment {
            user_id: val
                .user_id
                .as_deref()
                .and_then(parse_id)
                .unwrap_or_default(),
            collection_id: val
                .collection_id
                .as_deref()
                .and_then(parse_id)
                .unwrap_or_default(),
            role: val.role.as_str().into(),
        }
    }
}
