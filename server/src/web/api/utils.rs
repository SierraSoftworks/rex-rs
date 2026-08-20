use rex_api::ApiError;
use tracing::{info, instrument, warn};

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Collection, Role, RoleAssignment, User, format_id},
    services::Services,
};

/// Makes sure the caller exists as a user and owns their default collection.
///
/// Rex has always leaned on the invariant that a principal's id doubles as the
/// id of their default collection — it is what makes the v1 and v2 APIs, which
/// have no concept of collections at all, work. Every v3 endpoint that can be
/// the first thing a new user touches calls this first.
#[instrument(err, skip(services, token))]
pub async fn ensure_user_collection<S: Services>(
    services: &S,
    token: &AuthToken,
) -> Result<(), ApiError> {
    let uid = token.principal_id();

    if let Err(err) = services
        .store()
        .store_user(User {
            principal_id: uid,
            email_hash: token.email_hash(),
            first_name: token.first_name(),
        })
        .await
    {
        // Losing the users row costs the caller nothing right now — it only
        // affects whether other people can find them by email hash to share a
        // collection — so it is not worth failing the request over.
        warn!(
            "Unable to store an entry in the users table for this user: {}",
            err
        );
    }

    if let Err(err) = services.store().get_collection(uid, uid).await {
        info!(
            "User does not have a default collection ({}): {}",
            format_id(uid),
            err
        );

        services
            .store()
            .store_collection(Collection {
                collection_id: uid,
                user_id: uid,
                name: "My Ideas".into(),
            })
            .await?;
    }

    services
        .store()
        .store_role_assignment(RoleAssignment {
            collection_id: uid,
            user_id: uid,
            role: Role::Owner,
        })
        .await?;

    Ok(())
}
