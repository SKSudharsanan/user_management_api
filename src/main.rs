use actix_web::{web, App, HttpResponse, HttpServer, ResponseError};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

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

#[derive(Debug)]
enum ApiError {
    DbError(diesel::result::Error),
    PoolError(r2d2::PoolError),
    BlockingError(actix_web::error::BlockingError),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::DbError(e) => write!(f, "Database error: {}", e),
            ApiError::PoolError(e) => write!(f, "Pool error: {}", e),
            ApiError::BlockingError(e) => write!(f, "Blocking error: {}", e),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(format!("Internal Server Error: {}", self))
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(error: diesel::result::Error) -> Self {
        ApiError::DbError(error)
    }
}

impl From<r2d2::PoolError> for ApiError {
    fn from(error: r2d2::PoolError) -> Self {
        ApiError::PoolError(error)
    }
}

impl From<actix_web::error::BlockingError> for ApiError {
    fn from(error: actix_web::error::BlockingError) -> Self {
        ApiError::BlockingError(error)
    }
}

async fn create_user(pool: web::Data<DbPool>, new_user: web::Json<NewUser>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let user = web::block(move || {
        diesel::insert_into(users::table)
            .values(new_user.into_inner())
            .get_result::<User>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn get_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let user = web::block(move || {
        users::table.find(user_id.into_inner()).first::<User>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn update_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
    updated_user: web::Json<NewUser>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let user = web::block(move || {
        diesel::update(users::table.find(user_id.into_inner()))
            .set(updated_user.into_inner())
            .get_result::<User>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(user))
}

async fn delete_user(pool: web::Data<DbPool>, user_id: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
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
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
    .await
}