use std::vec::Vec;

use actix_web::{error, pred, App, HttpRequest, HttpResponse, Json, Path, Query, Result};

mod models;
mod state;
#[cfg(test)]
mod test;
use crate::stator::Stator;

pub use self::state::new_state;

pub fn configure(
    app: App<std::sync::Arc<::state::Container>>,
) -> App<std::sync::Arc<::state::Container>> {
    app.resource("/api/v1/ideas", |r| {
        r.route().filter(pred::Get()).with(ideas_v1);
        r.route().filter(pred::Post()).with(new_idea_v1);
    })
    .resource("/api/v1/idea/random", |r| {
        r.route().filter(pred::Get()).with(random_idea_v1)
    })
    .resource("/api/v1/idea/{id}", |r| {
        r.route().filter(pred::Get()).with(idea_v1);
        r.route().filter(pred::Put()).with(store_idea_v1);
    })
    .resource("/api/v2/ideas", |r| {
        r.route().filter(pred::Get()).with(ideas_v2);
        r.route().filter(pred::Post()).with(new_idea_v2);
    })
    .resource("/api/v2/idea/random", |r| {
        r.route().filter(pred::Get()).with(random_idea_v2);
    })
    .resource("/api/v2/idea/{id}", |r| {
        r.route().filter(pred::Get()).with(idea_v2);
        r.route().filter(pred::Put()).with(store_idea_v2);
    })
}

fn ideas_v1(state: Stator<state::IdeasState>) -> Result<Json<Vec<models::IdeaV1>>> {
    state::ideas(&state)
        .map(|val| Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

fn random_idea_v1(state: Stator<state::IdeasState>) -> Result<Json<models::IdeaV1>> {
    state::random_idea(|_| true, &state)
        .map(|val| Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

fn idea_v1(
    (info, state): (Path<IdFilter>, Stator<state::IdeasState>),
) -> Result<Json<models::IdeaV1>> {
    match u128::from_str_radix(&info.id, 16).ok() {
        Some(id) => state::idea(id, &state)
            .map(|val| Json(val))
            .ok_or(error::ErrorNotFound(
                "We could not find any idea with the ID you provided",
            )),
        None => Err(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
    }
}

fn new_idea_v1<S>(
    (mut new_idea, state, req): (
        Json<models::IdeaV1>,
        Stator<state::IdeasState>,
        HttpRequest<S>,
    ),
) -> Result<HttpResponse> {
    new_idea.id = None;

    let id = state::store_idea(&new_idea.into_inner(), &state).ok_or(
        error::ErrorInternalServerError(
            "We could not create a new idea with the details you provided",
        ),
    )?;

    Ok(HttpResponse::Created()
        .header(
            "Location",
            req.url_for("/api/v1/idea/{id}", &vec![format!("{:x}", id)])?
                .into_string(),
        )
        .json(state::idea::<models::IdeaV1>(id, &state)))
}

fn store_idea_v1(
    (info, mut new_idea, state): (
        Path<IdFilter>,
        Json<models::IdeaV1>,
        Stator<state::IdeasState>,
    ),
) -> Result<Json<models::IdeaV1>> {
    new_idea.id = Some(info.id.clone());

    match u128::from_str_radix(&info.id, 16).ok() {
        Some(_id) => {
            let id = state::store_idea(&new_idea.into_inner(), &state).ok_or(
                error::ErrorInternalServerError("We could not store the idea you provided"),
            )?;
            let idea = state::idea(id, &state).ok_or(error::ErrorNotFound(
                "We could not find an idea with the ID you provided",
            ))?;

            Ok(Json(idea))
        }
        None => Err(error::ErrorNotFound(
            "We could not find an idea with the ID you provided",
        )),
    }
}

fn ideas_v2(
    (query, state): (Query<QueryFilter>, Stator<state::IdeasState>),
) -> Result<Json<Vec<models::IdeaV2>>> {
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

    state::ideas_by(predicate, &state)
        .map(|val| Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

fn random_idea_v2(
    (query, state): (Query<QueryFilter>, Stator<state::IdeasState>),
) -> Result<Json<models::IdeaV2>> {
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

    state::random_idea(predicate, &state)
        .map(|val| Json(val))
        .ok_or(error::ErrorNotFound("We could not find any ideas"))
}

fn idea_v2(
    (info, state): (Path<IdFilter>, Stator<state::IdeasState>),
) -> Result<Json<models::IdeaV2>> {
    match u128::from_str_radix(&info.id, 16).ok() {
        Some(id) => state::idea(id, &state)
            .map(|val| Json(val))
            .ok_or(error::ErrorNotFound(
                "We could not find any idea with the ID you provided",
            )),
        None => Err(error::ErrorNotFound(
            "We could not find any idea with the ID you provided",
        )),
    }
}

fn new_idea_v2<S>(
    (mut new_idea, state, req): (
        Json<models::IdeaV2>,
        Stator<state::IdeasState>,
        HttpRequest<S>,
    ),
) -> Result<HttpResponse> {
    new_idea.id = None;

    let id = state::store_idea(&new_idea.into_inner(), &state).ok_or(
        error::ErrorInternalServerError(
            "We could not create a new idea with the details you provided",
        ),
    )?;

    Ok(HttpResponse::Created()
        .header(
            "Location",
            req.url_for("/api/v1/idea/{id}", &vec![format!("{:x}", id)])?
                .into_string(),
        )
        .json(state::idea::<models::IdeaV2>(id, &state)))
}

fn store_idea_v2(
    (info, mut new_idea, state): (
        Path<IdFilter>,
        Json<models::IdeaV2>,
        Stator<state::IdeasState>,
    ),
) -> Result<Json<models::IdeaV2>> {
    new_idea.id = Some(info.id.clone());

    match u128::from_str_radix(&info.id, 16).ok() {
        Some(_id) => {
            let id = state::store_idea(&new_idea.into_inner(), &state).ok_or(
                error::ErrorInternalServerError("We could not store the idea you provided"),
            )?;
            let idea = state::idea(id, &state).ok_or(error::ErrorNotFound(
                "We could not find an idea with the ID you provided",
            ))?;

            Ok(Json(idea))
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
