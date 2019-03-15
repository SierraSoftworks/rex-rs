use std::vec::Vec;

use rocket::response::status::Created;
use rocket::State;
use rocket_contrib::json::Json;

mod models;
mod state;
#[cfg(test)]
mod test;

pub use state::new_state;

#[get("/api/v1/ideas")]
pub fn ideas_v1(state: State<state::IdeasState>) -> Option<Json<Vec<models::IdeaV1>>> {
    state::ideas(state.inner()).map(|val| Json(val))
}

#[get("/api/v1/idea/random")]
pub fn random_idea_v1(state: State<state::IdeasState>) -> Option<Json<models::IdeaV1>> {
    state::random_idea(|_| true, state.inner()).map(|val| Json(val))
}

#[get("/api/v1/idea/<id>")]
pub fn idea_v1(id: String, state: State<state::IdeasState>) -> Option<Json<models::IdeaV1>> {
    match u128::from_str_radix(&id, 16).ok() {
        Some(id) => state::idea(id, state.inner()).map(|val| Json(val)),
        None => None,
    }
}

#[post("/api/v1/ideas", data = "<idea>")]
pub fn new_idea_v1(
    idea: Json<models::IdeaV1>,
    state: State<state::IdeasState>,
) -> Result<Created<Json<models::IdeaV1>>, rocket::http::Status> {
    let mut new_idea = idea.into_inner();
    new_idea.id = None;

    match state::store_idea(&new_idea, state.inner()) {
        Some(id) => Ok(Created(
            rocket::uri!(idea_v1: format!("{:x}", id)).to_string(),
            state::idea(id, state.inner()).map(|val| Json(val)),
        )),
        None => Err(rocket::http::Status::InternalServerError),
    }
}

#[put("/api/v1/idea/<id>", data = "<idea>")]
pub fn store_idea_v1(
    id: String,
    idea: Json<models::IdeaV1>,
    state: State<state::IdeasState>,
) -> Result<Json<models::IdeaV1>, rocket::http::Status> {
    let mut new_idea = idea.into_inner();
    new_idea.id = Some(id.clone());

    match u128::from_str_radix(&id, 16).ok() {
        Some(_id) => {
            match state::store_idea(&new_idea, state.inner()) {
                Some(id) => state::idea(id, state.inner()).map(|val| Json(val)).ok_or(rocket::http::Status::InternalServerError),
                None => Err(rocket::http::Status::InternalServerError),
            }
        },
        None => Err(rocket::http::Status::NotFound),
    }
}

#[get("/api/v2/ideas?<tag>&<complete>")]
pub fn ideas_v2(
    tag: Option<String>,
    complete: Option<bool>,
    state: State<state::IdeasState>,
) -> Option<Json<Vec<models::IdeaV2>>> {
    let predicate = |item: &models::Idea| {
        tag.clone()
            .map(|tag| item.tags.contains(&tag))
            .unwrap_or(true)
            && complete
                .clone()
                .map(|complete| item.completed == complete)
                .unwrap_or(true)
    };

    state::ideas_by(predicate, state.inner()).map(|val| Json(val))
}

#[get("/api/v2/idea/<id>")]
pub fn idea_v2(id: String, state: State<state::IdeasState>) -> Option<Json<models::IdeaV2>> {
    match u128::from_str_radix(&id, 16).ok() {
        Some(id) => state::idea(id, state.inner()).map(|val| Json(val)),
        None => None,
    }
}

#[post("/api/v2/ideas", data = "<idea>")]
pub fn new_idea_v2(
    idea: Json<models::IdeaV2>,
    state: State<state::IdeasState>,
) -> Result<Created<Json<models::IdeaV2>>, rocket::http::Status> {
    let mut new_idea = idea.into_inner();
    new_idea.id = None;

    match state::store_idea(&new_idea, state.inner()) {
        Some(id) => Ok(Created(
            rocket::uri!(idea_v2: format!("{:x}", id)).to_string(),
            state::idea(id, state.inner()).map(|val| Json(val)),
        )),
        None => Err(rocket::http::Status::InternalServerError),
    }
}

#[put("/api/v2/idea/<id>", data = "<idea>")]
pub fn store_idea_v2(
    id: String,
    idea: Json<models::IdeaV2>,
    state: State<state::IdeasState>,
) -> Result<Json<models::IdeaV2>, rocket::http::Status> {
    let mut new_idea = idea.into_inner();
    new_idea.id = Some(id.clone());

    match u128::from_str_radix(&id, 16).ok() {
        Some(_id) => {
            match state::store_idea(&new_idea, state.inner()) {
                Some(id) => state::idea(id, state.inner()).map(|val| Json(val)).ok_or(rocket::http::Status::InternalServerError),
                None => Err(rocket::http::Status::InternalServerError),
            }
        },
        None => Err(rocket::http::Status::NotFound),
    }
}

#[get("/api/v2/idea/random?<tag>&<complete>")]
pub fn random_idea_v2(
    tag: Option<String>,
    complete: Option<bool>,
    state: State<state::IdeasState>,
) -> Option<Json<models::IdeaV2>> {
    let predicate = |item: &models::Idea| {
        tag.clone()
            .map(|tag| item.tags.contains(&tag))
            .unwrap_or(true)
            && complete
                .clone()
                .map(|complete| item.completed == complete)
                .unwrap_or(true)
    };

    state::random_idea(predicate, state.inner()).map(|val| Json(val))
}
