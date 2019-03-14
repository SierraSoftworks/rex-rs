use super::super::app;
use super::models;
use super::state;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use serde_json::json;
use std::collections::HashSet;

macro_rules! hashset {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_set = HashSet::new();
            $(
                temp_set.insert($x);
            )*
            temp_set
        }
    };
}

#[test]
fn new_idea_v1() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    let mut response = client
        .post("/api/v1/ideas")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "Test Idea",
                "description": "This is a test idea",
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert!(response.headers().get_one("Location").is_some());
    assert!(response
        .headers()
        .get_one("Location")
        .unwrap()
        .starts_with("/api/v1/idea/"));

    let id =
        String::from(&response.headers().get_one("Location").unwrap()["/api/v1/idea/".len()..])
            .clone();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        1
    );

    let new_idea: models::IdeaV1 =
        serde_json::from_str(&response.body_string().expect("valid body"))
            .expect("valid json response");

    assert_eq!(new_idea.id, Some(id));
    assert_eq!(new_idea.name, "Test Idea".to_string());
    assert_eq!(new_idea.description, "This is a test idea".to_string());
}

#[test]
fn idea_v1() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    let id = state::new_idea(
        &models::IdeaV1 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        1
    );

    let mut response = client.get(format!("/api/v1/idea/{:x}", id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let idea: models::IdeaV1 = serde_json::from_str(&response.body_string().expect("valid body"))
        .expect("valid json response");

    assert_eq!(idea.id, Some(format!("{:x}", id)));
    assert_eq!(idea.name, "Test Idea".to_string());
    assert_eq!(idea.description, "This is a test idea".to_string());
}

#[test]
fn ideas_v1() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    state::new_idea(
        &models::IdeaV1 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
        },
        state,
    )
    .expect("create idea entry");

    state::new_idea(
        &models::IdeaV1 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        2
    );

    let mut response = client.get("/api/v1/ideas").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let ideas: Vec<models::IdeaV1> =
        serde_json::from_str(&response.body_string().expect("valid body"))
            .expect("valid json response");

    assert_eq!(ideas.len(), 2);

    for idea in ideas {
        assert_eq!(idea.name, "Test Idea".to_string());
        assert_eq!(idea.description, "This is a test idea".to_string());
    }
}

#[test]
fn new_idea_v2() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    let mut response = client
        .post("/api/v2/ideas")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "Test Idea",
                "description": "This is a test idea",
                "tags": ["test1", "test2"],
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert!(response.headers().get_one("Location").is_some());
    assert!(response
        .headers()
        .get_one("Location")
        .unwrap()
        .starts_with("/api/v2/idea/"));

    let id =
        String::from(&response.headers().get_one("Location").unwrap()["/api/v2/idea/".len()..])
            .clone();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        1
    );

    let new_idea: models::IdeaV2 =
        serde_json::from_str(&response.body_string().expect("valid body"))
            .expect("valid json response");

    assert_eq!(new_idea.id, Some(id));
    assert_eq!(new_idea.name, "Test Idea".to_string());
    assert_eq!(new_idea.description, "This is a test idea".to_string());
    assert!(new_idea.tags.contains(&"test1".to_string()));
    assert!(new_idea.tags.contains(&"test2".to_string()));
}

#[test]
fn idea_v2() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    let id = state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string(), "test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        1
    );

    let mut response = client.get(format!("/api/v2/idea/{:x}", id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let idea: models::IdeaV2 = serde_json::from_str(&response.body_string().expect("valid body"))
        .expect("valid json response");

    assert_eq!(idea.id, Some(format!("{:x}", id)));
    assert_eq!(idea.name, "Test Idea".to_string());
    assert_eq!(idea.description, "This is a test idea".to_string());
    assert!(idea.tags.contains(&"test1".to_string()));
    assert!(idea.tags.contains(&"test2".to_string()));
}

#[test]
fn random_idea_v2() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string(), "test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        1
    );

    let mut response = client.get("/api/v2/idea/random").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let idea: models::IdeaV2 = serde_json::from_str(&response.body_string().expect("valid body"))
        .expect("valid json response");

    assert_eq!(idea.name, "Test Idea".to_string());
    assert_eq!(idea.description, "This is a test idea".to_string());
    assert_eq!(idea.completed, Some(false));
    assert!(idea.tags.contains(&"test1".to_string()));
    assert!(idea.tags.contains(&"test2".to_string()));
}

#[test]
fn random_idea_v2_with_tags() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        2
    );

    for _ in 1..10 {
        let mut response = client.get("/api/v2/idea/random?tag=test1").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let idea: models::IdeaV2 =
            serde_json::from_str(&response.body_string().expect("valid body"))
                .expect("valid json response");

        assert_eq!(idea.name, "Test Idea".to_string());
        assert_eq!(idea.description, "This is a test idea".to_string());
        assert_eq!(idea.completed, Some(false));
        assert!(idea.tags.contains(&"test1".to_string()));
    }
}

#[test]
fn ideas_v2() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string(), "test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string(), "test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        2
    );

    let mut response = client.get("/api/v2/ideas").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let ideas: Vec<models::IdeaV2> =
        serde_json::from_str(&response.body_string().expect("valid body"))
            .expect("valid json response");

    assert_eq!(ideas.len(), 2);

    for idea in ideas {
        assert_eq!(idea.name, "Test Idea".to_string());
        assert_eq!(idea.description, "This is a test idea".to_string());
        assert_eq!(idea.completed, Some(false));
        assert!(idea.tags.contains(&"test1".to_string()));
        assert!(idea.tags.contains(&"test2".to_string()));
    }
}

#[test]
fn ideas_v2_with_tags() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::IdeasState = client.rocket().state().unwrap();

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        0
    );

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string(), "test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test2".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    state::new_idea(
        &models::IdeaV2 {
            id: None,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!["test1".to_string()],
            completed: None,
        },
        state,
    )
    .expect("create idea entry");

    assert_eq!(
        state
            .store
            .read()
            .expect("get read lock on the state")
            .len(),
        3
    );

    let mut response = client.get("/api/v2/ideas?tag=test1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let ideas: Vec<models::IdeaV2> =
        serde_json::from_str(&response.body_string().expect("valid body"))
            .expect("valid json response");

    assert_eq!(ideas.len(), 2);

    for idea in ideas {
        assert_eq!(idea.name, "Test Idea".to_string());
        assert_eq!(idea.description, "This is a test idea".to_string());
        assert_eq!(idea.completed, Some(false));
        assert!(idea.tags.contains(&"test1".to_string()));
    }
}
