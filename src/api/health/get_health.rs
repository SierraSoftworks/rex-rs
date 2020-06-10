use actix_web::{get, web};
use crate::models::*;
use crate::api::APIError;

#[get("/api/v1/health")]
pub async fn get_health_v1(state: web::Data<GlobalState>) -> Result<HealthV1, APIError> {
    state.store.send(GetHealth {}).await?.map(|health| health.clone().into())
}

#[get("/api/v2/health")]
pub async fn get_health_v2(state: web::Data<GlobalState>) ->Result<HealthV2, APIError> {
    state.store.send(GetHealth {}).await?.map(|health| health.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;
    
    #[actix_rt::test]
    async fn health_v1() {
        test_log_init();

        let content: HealthV1 = test_request!(GET "/api/v1/health" => OK with content);
        assert_eq!(content.ok, true);
    }

    #[actix_rt::test]
    async fn health_v2() {
        test_log_init();

        test_state!(state = []);

        let content: HealthV2 = test_request!(GET "/api/v2/health" => OK with content | state = state);
        assert_eq!(content.ok, true);
        assert_eq!(content.started_at, state.store.send(GetHealth {}).await.expect("the actor should respond").expect("we should get the health").started_at);
    }

}