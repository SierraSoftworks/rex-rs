use super::{APIError, AuthToken};
use crate::{models::*, telemetry::TraceMessageExt};

#[tracing::instrument(err, skip(state, token))]
pub async fn ensure_user_collection(
    state: &GlobalState,
    token: &AuthToken,
) -> Result<(), APIError> {
    let uid = u128::from_str_radix(token.oid().replace('-', "").as_str(), 16).map_err(|_| {
        APIError::new(
            400,
            "Bad Request",
            "The auth token OID you provided could not be parsed. Please check it and try again.",
        )
    })?;

    match state
        .store
        .send(
            StoreUser {
                principal_id: uid,
                email_hash: u128::from_be_bytes(
                    md5::compute(token.email().to_lowercase().trim().as_bytes()).into(),
                ),
                first_name: token.name().split(' ').next().unwrap_or("").to_string(),
            }
            .trace(),
        )
        .await?
    {
        Ok(_) => {}
        Err(err) => {
            warn!(
                "Unable to store an entry in the users table for this user: {}",
                err
            );
        }
    }

    match state
        .store
        .send(
            GetCollection {
                id: uid,
                principal_id: uid,
            }
            .trace(),
        )
        .await?
    {
        Ok(_) => {}
        Err(err) => {
            info!(
                "User does not have a default collection ({:0>32x}): {}",
                uid, err
            );

            state
                .store
                .send(
                    StoreCollection {
                        collection_id: uid,
                        principal_id: uid,
                        name: "My Ideas".into(),
                    }
                    .trace(),
                )
                .await??;
        }
    }

    state
        .store
        .send(
            StoreRoleAssignment {
                collection_id: uid,
                principal_id: uid,
                role: Role::Owner,
            }
            .trace(),
        )
        .await??;

    Ok(())
}
