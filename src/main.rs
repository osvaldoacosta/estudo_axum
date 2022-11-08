use axum::{  Extension,async_trait,http::{request::Parts, StatusCode},
    routing::get,routing::post, Router,response::IntoResponse, Json, extract::Path};
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::{Any,CorsLayer};
mod types;
use types::Person;
use types::PersonReq;
use sqlx::postgres::{PgPool, PgPoolOptions};





fn internal_error<E>(err: E) -> (StatusCode, String) where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    //Adiciona a regra do cors, nesse caso vai set todo permissivo
    let cors = CorsLayer::new().allow_origin(Any);
    //Seta as rotas do app(endpoints) 

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/estudo?user=dinho&password=741".to_string());   

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_str)
        .await
        .expect("can connect to database"); 
    
    let app = Router::new()
        .route("/", get(greetings))
        .route("/people", get(get_people).post(create_people)) 
        .route("/people/:id", get(get_person).delete(delete_people).put(update_people))
        .layer(cors)
        .layer(Extension(pool));
    //Seta o endereco do web socket onde vai ser iniciado o server
    let addr = SocketAddr::from(([127,0,0,1],3000));
    tracing::info!("listeninig on port: {}", addr);
     
    //Da o bind dos endpoints no endereço do socket
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");

   

}

async fn greetings() -> &'static str{
    "Hello axum"
}

async fn get_people(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    // add code here

    let people:Result<Vec<Person>,_>= sqlx::query_as!(Person,"SELECT * FROM person") 
        .fetch_all(&pool)
        .await;
   
    let people_vec:Vec<Person> = people.unwrap();
    (StatusCode::OK, Json(people_vec))

}


async fn get_person(Extension(pool): Extension<PgPool>,Path(id) : Path<i32>) ->  Result<axum::Json<Person>, (StatusCode, String)>{
    sqlx::query_as!(
        Person,"select * from person where id = $1", id
    )
    .fetch_one(&pool)
    .await
    .map(|person| axum::Json(person))
    .map_err(internal_error)
}

async fn create_people(Json(payload):Json<PersonReq>,Extension(pool): Extension<PgPool>) -> impl IntoResponse{
    sqlx::query!("INSERT INTO person (name, age) VALUES ($1, $2)", 
        payload.name, payload.age)
        .execute(&pool)
        .await
        .map(|resp| (StatusCode::OK, format!("Person created - {:?}", resp)))
        .map_err(internal_error)
    
}


async fn update_people(Json(payload):Json<PersonReq>, Extension(pool): Extension<PgPool>, Path(id) : Path<i32>)->impl IntoResponse{
    let person = sqlx::query_as!(
        Person,"select * from person where id = $1", id
    )
    .fetch_one(&pool)
    .await
    .map_err(|resp| (StatusCode::NO_CONTENT, false))
    .map(|resp| (StatusCode::OK, true));

    if !person.unwrap().1 {
        return (person.unwrap().0, String::from("Id not found"));
    } 
        sqlx::query!("UPDATE person SET name=$1,age=$2 WHERE id=$3;", 
        payload.name, payload.age, id)
        .execute(&pool)
        .await
        .map(|resp| (StatusCode::OK, format!("Person updated - {:?}", resp)))
        .map_err(internal_error).unwrap()
   
}
async fn delete_people(Path(id) : Path<i32>, Extension(pool): Extension<PgPool>)-> impl IntoResponse{
    let person = sqlx::query_as!(
        Person,"select * from person where id = $1", id
    )
    .fetch_one(&pool)
    .await
    .map_err(|resp| (StatusCode::NO_CONTENT, false))
    .map(|resp| (StatusCode::OK, true));

    if !person.unwrap().1 {
        return (person.unwrap().0, String::from("Id not found"));
    } 

    sqlx::query!("DELETE FROM person WHERE id=$1", 
        id)
        .execute(&pool)
        .await
        .map_err(internal_error)
        .map(|res| (StatusCode::OK, String::from("Person deleted"))).unwrap()
        
    
}

