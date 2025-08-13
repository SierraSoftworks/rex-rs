use crate::api::APIError;
use actix::prelude::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Role {
    Owner,
    Contributor,
    #[default]
    Viewer,
    Invalid,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub user_id: u128,
    pub collection_id: u128,
    pub role: Role,
}

actor_message!(GetRoleAssignment(collection_id: u128, principal_id: u128) -> RoleAssignment);

actor_message!(GetRoleAssignments(collection_id: u128) -> Vec<RoleAssignment>);

actor_message!(StoreRoleAssignment(collection_id: u128, principal_id: u128, role: Role) -> RoleAssignment);

actor_message!(RemoveRoleAssignment(collection_id: u128, principal_id: u128) -> ());

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleAssignmentV3 {
    #[serde(rename = "collectionId")]
    pub collection_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub role: String,
}

json_responder!(RoleAssignmentV3 => (req, model) -> req.url_for("get_role_assignment_v3", vec![
    model.collection_id.clone().expect("a collection id"),
    model.user_id.clone().expect("a user id")
]));

impl From<RoleAssignment> for RoleAssignmentV3 {
    fn from(idea: RoleAssignment) -> Self {
        Self {
            user_id: Some(format!("{:0>32x}", idea.user_id)),
            collection_id: Some(format!("{:0>32x}", idea.collection_id)),
            role: idea.role.into(),
        }
    }
}

impl From<RoleAssignmentV3> for RoleAssignment {
    fn from(val: RoleAssignmentV3) -> Self {
        RoleAssignment {
            user_id: val
                .user_id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            collection_id: val
                .collection_id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            role: val.role.as_str().into(),
        }
    }
}
