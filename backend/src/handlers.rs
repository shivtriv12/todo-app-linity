//// filepath: /home/shivtriv/todo-app-linity/backend/src/handlers.rs
use axum::{
    extract::{Extension,Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Pool;
use sqlx::Postgres;
use crate::models::{Todo, NewTodo,UpdateTodo};

pub async fn get_all_todos_handler(
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Fetch all todos from the database and map errors to a proper status code.
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, completed, user_id FROM todos")
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", err),
            )
        })?;

    Ok(Json(todos))
}

pub async fn create_todo_handler(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(payload): Json<NewTodo>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let inserted_todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed, user_id) VALUES ($1, false, $2) RETURNING id, title, completed, user_id"
    )
    .bind(payload.title)
    .bind(payload.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB Error: {}", err),
        )
    })?;
    
    Ok(Json(inserted_todo))
}

pub async fn update_todo_handler(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTodo>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Using COALESCE to update only fields provided; if not provided, keep existing values.
    let updated_todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = COALESCE($1, title), completed = COALESCE($2, completed)
         WHERE id = $3
         RETURNING id, title, completed, user_id"
    )
    .bind(payload.title)
    .bind(payload.completed)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|err| (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("DB Error: {}", err),
    ))?;

    Ok(Json(updated_todo))
}

pub async fn delete_todo_handler(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Execute a DELETE query on the todos table for the given id.
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", err),
            )
        })?;

    // Check if any row was deleted; if not, return a 404 error.
    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            format!("Todo with ID {} not found", id),
        ))
    } else {
        // Return 204 No Content on success.
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn get_todo_handler(
    Path(id): Path<i32>, // use i32 instead of u32
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let todo = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, user_id FROM todos WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", err),
        )
    })?;

    if let Some(todo) = todo {
        Ok(Json(todo))
    } else {
        Err((StatusCode::NOT_FOUND, format!("Todo with id {} not found", id)))
    }
}