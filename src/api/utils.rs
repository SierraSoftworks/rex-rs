use crate::models::*;
use super::{AuthToken, APIError};

pub async fn ensure_user_collection(state: &GlobalState, token: &AuthToken) -> Result<(), APIError> {
    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    match state.store.send(GetCollection {
        id: uid,
        principal_id: uid,
    }).await? {
        Ok(_) => {},
        Err(err) => {
            info!("User does not have a default collection ({}): {}", uid, err);

            state.store.send(StoreCollection {
                collection_id: uid,
                principal_id: uid,
                name: "My Ideas".into(),
            }).await??;
        }
    }

    match state.store.send(GetRoleAssignment {
        collection_id: uid,
        principal_id: uid,
    }).await? {
        Ok(_) => {},
        Err(err) => {
            info!("User does not have a default role assignment for their default collection ({}): {}", uid, err);

            state.store.send(StoreRoleAssignment {
                collection_id: uid,
                principal_id: uid,
                role: Role::Owner,
            }).await??;
        }
    }

    Ok(())
}