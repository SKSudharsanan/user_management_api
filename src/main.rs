use actix_web::{web, App, HttpResponse, HttpServer, ResponseError};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

mod schema;
use schema::users;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Queryable, Serialize)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Deserialize)]
#[diesel(table_name = users)]
struct NewUser {
    username: String,
    email: String,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Database error: {0}")]
    DbError(#[from] diesel::result::Error),
    #[error("Environment error: {0}")]
    EnvError(#[from] std::env::VarError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Blocking error: {0}")]
    BlockingError(#[from] actix_web::error::BlockingError),
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(format!("Internal Server Error: {}", self))
    }
}

async fn create_user(pool: web::Data<DbPool>, new_user: web::Json<NewUser>) -> Result<HttpResponse, MyError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let user = web::block(move || {
        diesel::insert_into(users::table)
            .values(new_user.into_inner())
            .get_result::<User>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn get_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<HttpResponse, MyError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let user = web::block(move || users::table.find(user_id.into_inner()).first::<User>(&mut conn))
        .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn update_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
    updated_user: web::Json<NewUser>,
) -> Result<HttpResponse, MyError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let user = web::block(move || {
        diesel::update(users::table.find(user_id.into_inner()))
            .set(updated_user.into_inner())
            .get_result::<User>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn delete_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<HttpResponse, MyError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let deleted = web::block(move || {
        diesel::delete(users::table.find(user_id.into_inner())).execute(&mut conn)
    })
    .await??;

    if deleted > 0 {
        Ok(HttpResponse::Ok().body("User deleted successfully"))
    } else {
        Ok(HttpResponse::NotFound().body("User not found"))
    }
}

#[actix_web::main]
async fn main() -> Result<(), MyError> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}