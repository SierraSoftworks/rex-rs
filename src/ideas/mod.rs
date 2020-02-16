use std::vec::Vec;

use actix_web::{error, get, post, put, web, Error, HttpRequest, HttpResponse};

mod models;
mod state;
#[cfg(test)]
mod test;

pub use self::state::IdeasState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(ideas_v1)
        .service(random_idea_v1)
        .service(idea_v1)
        .service(new_idea_v1)
        .service(store_idea_v1);

    cfg.service(ideas_v2)
        .service(random_idea_v2)
        .service(idea_v2)
        .service(new_idea_v2)
        .service(store_idea_v2);
}

#[get("/api/v1/ideas")]
async fn ideas_v1(
    state: web::Data<state::IdeasState>,
) -> Result<web::Json<Vec<models::IdeaV1>>, Error> {
    state
        .ideas()
        .map(|val| web::Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

#[get("/api/v1/idea/random")]
async fn random_idea_v1(state: web::Data<state::IdeasState>) -> Result<models::IdeaV1, Error> {
    state
        .random_idea(|_| true)
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

#[get("/api/v1/idea/{id}")]
async fn idea_v1(
    (info, state): (web::Path<IdFilter>, web::Data<state::IdeasState>),
) -> Result<models::IdeaV1, Error> {
    match u128::from_str_radix(&info.id, 16).ok() {
        Some(id) => state.idea(id).ok_or(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
        None => Err(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
    }
}

#[post("/api/v1/ideas")]
async fn new_idea_v1(
    (mut new_idea, state, req): (
        web::Json<models::IdeaV1>,
        web::Data<state::IdeasState>,
        HttpRequest,
    ),
) -> Result<models::IdeaV1, Error> {
    new_idea.id = None;

    state
        .store_idea(&new_idea.into_inner())
        .ok_or(error::ErrorInternalServerError(
            "We could not create a new idea with the details you provided",
        ))
}

#[put("/api/v1/idea/{id}")]
async fn store_idea_v1(
    (info, mut new_idea, state): (
        web::Path<IdFilter>,
        web::Json<models::IdeaV1>,
        web::Data<state::IdeasState>,
    ),
) -> Result<models::IdeaV1, Error> {
    new_idea.id = Some(info.id.clone());

    match u128::from_str_radix(&info.id, 16).ok() {
        Some(_id) => state
            .store_idea(&new_idea.into_inner())
            .ok_or(error::ErrorNotFound(
                "We could not find an idea with the ID you provided",
            )),
        None => Err(error::ErrorNotFound(
            "We could not find an idea with the ID you provided",
        )),
    }
}

#[get("/api/v2/ideas")]
async fn ideas_v2(
    (query, state): (web::Query<QueryFilter>, web::Data<state::IdeasState>),
) -> Result<web::Json<Vec<models::IdeaV2>>, Error> {
    let predicate = |item: &models::Idea| {
        query
            .tag
            .clone()
            .map(|tag| item.tags.contains(&tag))
            .unwrap_or(true)
            && query
                .complete
                .clone()
                .map(|complete| item.completed == complete)
                .unwrap_or(true)
    };

    state
        .ideas_by(predicate)
        .map(|val| web::Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

#[get("/api/v2/idea/random")]
async fn random_idea_v2(
    (query, state): (web::Query<QueryFilter>, web::Data<state::IdeasState>),
) -> Result<models::IdeaV2, Error> {
    let predicate = |item: &models::Idea| {
        query
            .tag
            .clone()
            .map(|tag| item.tags.contains(&tag))
            .unwrap_or(true)
            && query
                .complete
                .clone()
                .map(|complete| item.completed == complete)
                .unwrap_or(true)
    };

    state
        .random_idea(predicate)
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

#[get("/api/v2/idea/{id}")]
async fn idea_v2(
    (info, state): (web::Path<IdFilter>, web::Data<state::IdeasState>),
) -> Result<models::IdeaV2, Error> {
    match u128::from_str_radix(&info.id, 16).ok() {
        Some(id) => state.idea(id).ok_or(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
        None => Err(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
    }
}

#[post("/api/v2/ideas")]
async fn new_idea_v2(
    (mut new_idea, state, req): (
        web::Json<models::IdeaV2>,
        web::Data<state::IdeasState>,
        HttpRequest,
    ),
) -> Result<models::IdeaV2, Error> {
    new_idea.id = None;

    state
        .store_idea(&new_idea.into_inner())
        .ok_or(error::ErrorInternalServerError(
            "We could not create a new idea with the details you provided",
        ))
}

#[put("/api/v2/idea/{id}")]
async fn store_idea_v2(
    (info, mut new_idea, state): (
        web::Path<IdFilter>,
        web::Json<models::IdeaV2>,
        web::Data<state::IdeasState>,
    ),
) -> Result<models::IdeaV2, Error> {
    new_idea.id = Some(info.id.clone());

    match u128::from_str_radix(&info.id, 16).ok() {
        Some(_id) => {
            state
                .store_idea(&new_idea.into_inner())
                .ok_or(error::ErrorInternalServerError(
                    "We could not find an idea with the ID you provided",
                ))
        }
        None => Err(error::ErrorNotFound(
            "We could not find an idea with the ID you provided",
        )),
    }
}

#[derive(Deserialize, Serialize)]
struct IdFilter {
    id: String,
}

#[derive(Deserialize)]
pub struct QueryFilter {
    tag: Option<String>,
    complete: Option<bool>,
}
