use super::UserFilter;
use super::{APIError, AuthToken};
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{get, web};
use tracing::instrument;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/user/{user}")]
async fn get_user_v3(
    (info, state, token): (web::Path<UserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<UserV3, APIError> {
    require_role!(token, "Administrator", "User");

    let tuid = parse_uuid!(info.user, "user ID");

    state
        .store
        .send(GetUser { email_hash: tuid }.trace())
        .await?
        .map(|user| user.into())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn get_user_v3() {
        test_log_init();

        test_state!(
            state = [StoreUser {
                email_hash: 1,
                principal_id: 0,
                first_name: "Test".to_string(),
            }]
        );

        let content: UserV3 = test_request!(GET "/api/v3/user/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(
            content.email_hash,
            "00000000000000000000000000000001".to_string()
        );
        assert_eq!(content.id, "00000000000000000000000000000000".to_string());
        assert_eq!(content.first_name, "Test".to_string());
    }
}
