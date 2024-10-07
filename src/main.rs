use mongodb::{options::ClientOptions, Client};
use std::env;
use dotenv::dotenv;

use actix_web::{web, App, HttpServer};

mod course;
mod unit;
mod class;
mod users;
mod comment;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    dotenv().ok();
    
    let db_uri: String = env::var("MONGODB_URI").expect("Expected MONGODB_URI in env");
    let client_options: ClientOptions = ClientOptions::parse(db_uri).await?;
    let client: Client = Client::with_options(client_options)?;
    println!("Connected to MongoDB!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    
    Ok(())
}