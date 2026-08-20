/// Builds a `HashSet` from a list of values, or from anything iterable when
/// written as `hashset!([values])`.
#[macro_export]
macro_rules! hashset {
    ([$x:expr]) => {{
        let mut set = std::collections::HashSet::new();
        for value in $x {
            set.insert(value.into());
        }
        set
    }};

    ( $( $x:expr ),* ) => {{
        let mut set = std::collections::HashSet::new();
        $( set.insert($x.into()); )*
        set
    }};
}

/// Builds a test services container seeded with the entities a case needs.
///
/// ```ignore
/// test_state!(state = [
///     collection { collection_id: 0, user_id: 0, name: "My Ideas".into() },
///     idea { id: 1, collection_id: 0, name: "Test Idea".into(), ..Default::default() },
/// ]);
/// ```
///
/// A `collection` seed also grants its `user_id` the Owner role, because that
/// is how every collection is really created, and because ideas and role
/// assignments now require a collection that exists.
#[macro_export]
macro_rules! test_state {
    ($state:ident = [ $($kind:ident { $($body:tt)* }),* $(,)? ]) => {
        let $state = $crate::testing::test_services();

        $( $crate::test_state!(@seed $state, $kind { $($body)* }); )*
    };

    (@seed $state:ident, collection { $($body:tt)* }) => {{
        use $crate::db::Store as _;
        use $crate::services::Services as _;

        let collection = $crate::models::Collection { $($body)* };

        $state
            .store()
            .store_collection(collection.clone())
            .await
            .expect("the collection should seed");

        $state
            .store()
            .store_role_assignment($crate::models::RoleAssignment {
                collection_id: collection.collection_id,
                user_id: collection.user_id,
                role: $crate::models::Role::Owner,
            })
            .await
            .expect("the collection owner should seed");
    }};

    (@seed $state:ident, role { $($body:tt)* }) => {{
        use $crate::db::Store as _;
        use $crate::services::Services as _;

        $state
            .store()
            .store_role_assignment($crate::models::RoleAssignment { $($body)* })
            .await
            .expect("the role assignment should seed");
    }};

    (@seed $state:ident, idea { $($body:tt)* }) => {{
        use $crate::db::Store as _;
        use $crate::services::Services as _;

        $state
            .store()
            .store_idea($crate::models::Idea { $($body)* })
            .await
            .expect("the idea should seed");
    }};

    (@seed $state:ident, user { $($body:tt)* }) => {{
        use $crate::db::Store as _;
        use $crate::services::Services as _;

        $state
            .store()
            .store_user($crate::models::User { $($body)* })
            .await
            .expect("the user should seed");
    }};
}

/// Drives a request through the real application factory and asserts its status.
#[macro_export]
macro_rules! test_request {
    ($method:ident $path:expr => $status:ident | state = $state:ident) => {{
        let app = $crate::testing::get_test_app($state.clone()).await;
        let req = actix_web::test::TestRequest::with_uri($path)
            .method(actix_web::http::Method::$method)
            .insert_header(("Authorization", $crate::testing::auth_token()))
            .to_request();

        let response = actix_web::test::call_service(&app, req).await;
        $crate::testing::assert_status(response, actix_web::http::StatusCode::$status).await
    }};

    ($method:ident $path:expr, $body:expr => $status:ident | state = $state:ident) => {{
        let app = $crate::testing::get_test_app($state.clone()).await;
        let req = actix_web::test::TestRequest::with_uri($path)
            .method(actix_web::http::Method::$method)
            .set_json(&$body)
            .insert_header(("Authorization", $crate::testing::auth_token()))
            .to_request();

        let response = actix_web::test::call_service(&app, req).await;
        $crate::testing::assert_status(response, actix_web::http::StatusCode::$status).await
    }};

    ($method:ident $path:expr => $status:ident with content | state = $state:ident) => {{
        let response = $crate::test_request!($method $path => $status | state = $state);
        $crate::testing::get_content(response).await
    }};

    ($method:ident $path:expr, $body:expr => $status:ident with content | state = $state:ident) => {{
        let response = $crate::test_request!($method $path, $body => $status | state = $state);
        $crate::testing::get_content(response).await
    }};

    ($method:ident $path:expr => $status:ident with location =~ $location:expr, content | state = $state:ident) => {{
        let response = $crate::test_request!($method $path => $status | state = $state);
        $crate::testing::assert_location_header(response.headers(), $location);
        $crate::testing::get_content(response).await
    }};

    ($method:ident $path:expr, $body:expr => $status:ident with location =~ $location:expr, content | state = $state:ident) => {{
        let response = $crate::test_request!($method $path, $body => $status | state = $state);
        $crate::testing::assert_location_header(response.headers(), $location);
        $crate::testing::get_content(response).await
    }};

    /* ---------- forms which build their own empty state ---------- */

    ($method:ident $path:expr => $status:ident) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path => $status | state = state)
    }};

    ($method:ident $path:expr, $body:expr => $status:ident) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path, $body => $status | state = state)
    }};

    ($method:ident $path:expr => $status:ident with content) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path => $status with content | state = state)
    }};

    ($method:ident $path:expr, $body:expr => $status:ident with content) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path, $body => $status with content | state = state)
    }};

    ($method:ident $path:expr => $status:ident with location =~ $location:expr, content) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path => $status with location =~ $location, content | state = state)
    }};

    ($method:ident $path:expr, $body:expr => $status:ident with location =~ $location:expr, content) => {{
        let state = $crate::testing::test_services();
        $crate::test_request!($method $path, $body => $status with location =~ $location, content | state = state)
    }};
}
