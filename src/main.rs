#![deny(warnings)]
use std::env;
use warp::Filter;

mod types;
mod handlers;
mod paths;

/// Provides a RESTful web server managing some Todos.
///
/// API will be:
///
/// - `GET /todos`: return a JSON list of Todos.
/// - `POST /todos`: create a new Todo.
/// - `PUT /todos/:id`: update a specific Todo.
/// - `DELETE /todos/:id`: delete a specific Todo.
#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "todos=info");
    }
    pretty_env_logger::init();

    let db = types::blank_db();

    let api = paths::todos(db);

    // View access logs by setting `RUST_LOG=todos`.
    let routes = api.with(warp::log("todos"));
    // Start up the server...
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}



#[cfg(test)]
mod tests {
    use warp::http::StatusCode;
    use warp::test::request;

    use super::{
        paths,
        types::{self, Todo},
    };

    #[tokio::test]
    async fn test_post() {
        let db = types::blank_db();
        let api = paths::todos(db);

        let resp = request()
            .method("POST")
            .path("/todos")
            .json(&todo1())
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_post_conflict() {
        let db = types::blank_db();
        db.lock().await.push(todo1());
        let api = paths::todos(db);

        let resp = request()
            .method("POST")
            .path("/todos")
            .json(&todo1())
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_put_unknown() {
        let _ = pretty_env_logger::try_init();
        let db = types::blank_db();
        let api = paths::todos(db);

        let resp = request()
            .method("PUT")
            .path("/todos/1")
            .header("authorization", "Bearer admin")
            .json(&todo1())
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    fn todo1() -> Todo {
        Todo {
            id: 1,
            text: "test 1".into(),
            completed: false,
        }
    }
}