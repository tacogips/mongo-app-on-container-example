mod service;

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Json, Router, Server,
};
use service::*;
use std::{net::SocketAddr, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

macro_rules! rest_layer {
    () => {
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|error: BoxError| async move {
                if error.is::<tower::timeout::error::Elapsed>() {
                    Ok(StatusCode::REQUEST_TIMEOUT)
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("unhandlesd internal error :{}", error),
                    ))
                }
            }))
            .timeout(Duration::from_secs(20))
            .layer(TraceLayer::new_for_http())
            .into_inner()
    };
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let todo_service = TodoService::default();

    let app = Router::new()
        .route("/api/todos", get(find_todos).post(new_todo))
        .route(
            "/api/todos/:todo_id",
            get(get_todo)
                .put(edit_todo_text)
                .delete(remove_todo)
                .layer(rest_layer!()),
        )
        .route(
            "/api/todos/:todo_id/status",
            get(get_todo).put(update_todo_status),
        )
        .fallback(get_service(ServeDir::new("./public")).handle_error(handle_static_file_error))
        .layer(rest_layer!())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_headers(AllowHeaders::any())
                .allow_methods(AllowMethods::any()),
        )
        .layer(Extension(todo_service));

    //TODO(tacogips) add SSE case with mongo stream

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    info!("server listengin at :5000");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn find_todos(
    Extension(todo_service): Extension<TodoService>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = todo_service
        .find()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

async fn new_todo(
    Extension(todo_service): Extension<TodoService>,
    Json(todo): Json<Todo>,
) -> Result<Json<Todo>, StatusCode> {
    info!("new todo");

    let todo = todo_service
        .create(todo)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todo))
}

fn parse_todo_id(id: &str) -> Result<TodoId, StatusCode> {
    let todo_id = todo_id_from_str(id).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(todo_id)
}

async fn get_todo(
    Path(todo_id): Path<String>,
    Extension(todo_service): Extension<TodoService>,
) -> Result<Json<Todo>, StatusCode> {
    info!("get todo {todo_id}");

    let todo_id = parse_todo_id(&todo_id)?;
    let todo = todo_service
        .get(todo_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match todo {
        Some(todo) => Ok(Json(todo)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn edit_todo_text(
    Path(todo_id): Path<String>,
    Json(update): Json<UpdateTodoTextRequest>,
    Extension(todo_service): Extension<TodoService>,
) -> StatusCode {
    info!("edit todo {todo_id}");

    let todo_id = parse_todo_id(&todo_id);
    let todo_id = match todo_id {
        Ok(id) => id,
        Err(s) => return s,
    };
    let todo = todo_service.update_text(todo_id, update).await;

    match todo {
        Err(e) => {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(todo) => match todo {
            Some(_) => StatusCode::OK,
            None => StatusCode::NOT_FOUND,
        },
    }
}

async fn update_todo_status(
    Path(todo_id): Path<String>,
    Json(update): Json<UpdateTodoStatusRequest>,
    Extension(todo_service): Extension<TodoService>,
) -> StatusCode {
    info!("edit todo status {todo_id}");

    let todo_id = parse_todo_id(&todo_id);
    let todo_id = match todo_id {
        Ok(id) => id,
        Err(s) => return s,
    };

    let todo = todo_service.update_status(todo_id, update).await;

    match todo {
        Err(e) => {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(todo) => match todo {
            Some(_) => StatusCode::OK,
            None => StatusCode::NOT_FOUND,
        },
    }
}

async fn remove_todo(
    Path(todo_id): Path<String>,
    Extension(todo_service): Extension<TodoService>,
) -> StatusCode {
    info!("remove TODO {todo_id:?}");

    let todo_id = parse_todo_id(&todo_id);
    let todo_id = match todo_id {
        Ok(id) => id,
        Err(s) => return s,
    };

    let todo = todo_service.remove(todo_id).await;

    match todo {
        Err(e) => {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(todo) => match todo {
            Some(_) => StatusCode::OK,
            None => StatusCode::NOT_FOUND,
        },
    }
}

async fn handle_static_file_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "file not found")
}
