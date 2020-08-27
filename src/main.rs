#[macro_use]
extern crate diesel;

use actix_web::{delete, get, web, middleware, put, post, App, Error, HttpRequest, HttpResponse, HttpServer, FromRequest, Responder};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use listenfd::ListenFd;
use log::{info, trace, warn};

mod models;
mod schema;
mod actions;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// Handler functions here
async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

/// Find person by id
#[get("/person/{person_id}")]
async fn get_person(
    pool: web::Data<DbPool>,
    person_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let person_id = person_id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let person = web::block(move || actions::find_person_by_id(person_id, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(person) = person {
        Ok(HttpResponse::Ok().json(person))
    } else {
        let res = HttpResponse::NotFound()
            .body(format!("No user found with id: {}", person_id));
        Ok(res)
    }
}

/// Get all persons
#[get("/persons")]
async fn get_person_list(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let persons = web::block(move || actions::get_all_persons(&conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::Ok().json(persons))
}

/// Inserts new person with name defined in form
#[post("/person")]
async fn create_person(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewPerson>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let person = web::block(move || actions::insert_new_person(&form.name, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(person))
}

/// Update a person
#[put("/person/{person_id}")]
async fn update_person(
    pool: web::Data<DbPool>,
    person_id: web::Path<i32>,
    form: web::Json<models::UpdatePerson>,
) -> Result<HttpResponse, Error> {
    let person_id = person_id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");
    web::block(move || actions::update_person(person_id, &form.name, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().body(format!("Person {} updated.", person_id)))
}

/// Delete a person
#[delete("/person/{person_id}")]
async fn delete_person(
    pool: web::Data<DbPool>,
    person_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let person_id = person_id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    web::block(move || actions::delete_person(person_id, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::Ok().body(format!("Person {} deleted.", person_id)))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(get_person)
            .service(create_person)
            .service(get_person_list)
            .service(update_person)
            .service(delete_person)
            // .service(web::resource("/persons")
            //    .route(web::get().to(get_person_list)))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };
        
    server.run().await
}
