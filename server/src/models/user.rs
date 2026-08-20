use rex_api::UserV3;

use super::{Id, format_id, parse_id};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct User {
    pub principal_id: Id,
    pub email_hash: Id,
    pub first_name: String,
}

impl From<User> for UserV3 {
    fn from(user: User) -> Self {
        Self {
            id: format_id(user.principal_id),
            email_hash: format_id(user.email_hash),
            first_name: user.first_name,
        }
    }
}

impl From<UserV3> for User {
    fn from(val: UserV3) -> Self {
        User {
            principal_id: parse_id(&val.id).unwrap_or_default(),
            email_hash: parse_id(&val.email_hash).unwrap_or_default(),
            first_name: val.first_name,
        }
    }
}
