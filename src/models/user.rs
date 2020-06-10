use actix::prelude::*;
use crate::api::APIError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub principal_id: u128,
    pub email_hash: u128,
    pub first_name: String,
}

actor_message!(GetUser(email_hash: u128) -> User);

actor_message!(StoreUser(email_hash: u128, principal_id: u128, first_name: String) -> User);

#[derive(Debug, Serialize, Deserialize)]
pub struct UserV3 {
    pub id: String,

    #[serde(rename = "emailHash")]
    pub email_hash: String,

    #[serde(rename = "firstName")]
    pub first_name: String,
}

json_responder!(UserV3 => (req, model) -> req.url_for("get_user_v3", &vec![
    model.email_hash.clone()
]));

impl From<User> for UserV3 {
    fn from(user: User) -> Self {
        Self {
            id: format!("{:0>32x}", user.principal_id),
            email_hash: format!("{:0>32x}", user.email_hash),
            first_name: user.first_name.into(),
        }
    }
}

impl Into<User> for UserV3 {
    fn into(self) -> User {
        User {
            principal_id: u128::from_str_radix(&self.id, 16).unwrap_or_else(|_| 0),
            email_hash: u128::from_str_radix(&self.email_hash, 16).unwrap_or(0),
            first_name: self.first_name.as_str().into(),
        }
    }
}
