use actix_web::web;
use rex_api::{ApiError, UserV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::parse_id_or_400,
    services::Services,
    web::{ApiResponse, api::UserFilter},
};

/// Looks a user up by the MD5 of their email address.
///
/// This is how the invite flow turns an address somebody typed into a principal
/// id without either party learning anything they did not already know. The
/// hash is a lookup key and nothing more — no authorization decision is derived
/// from it.
#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_user_v3<S: Services>(
    info: web::Path<UserFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<UserV3>, ApiError> {
    let _ = &token;
    let hash = parse_id_or_400(&info.user, "user ID")?;

    services
        .store()
        .get_user(hash)
        .await
        .map(|user| ApiResponse(user.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::UserV3;

    use crate::{test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_user_v3() {
        test_log_init();

        test_state!(
            state = [user {
                email_hash: 1,
                principal_id: 0,
                first_name: "Test".to_string()
            },]
        );

        let content: UserV3 = test_request!(GET "/api/v3/user/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(
            content.email_hash,
            "00000000000000000000000000000001".to_string()
        );
        assert_eq!(content.id, "00000000000000000000000000000000".to_string());
        assert_eq!(content.first_name, "Test".to_string());
    }

    #[actix_rt::test]
    async fn get_user_v3_unknown() {
        test_log_init();

        test_request!(GET "/api/v3/user/000000000000000000000000000000ff" => NOT_FOUND);
    }
}
