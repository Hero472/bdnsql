use clap::{command, Parser};
use class::ClassyReceive;
use course::{create_complete_course, FullCourse};
use mongodb::{options::ClientOptions, Client};
use routes::{classy_config, comment_config, courses_config, unit_config, user_config};
use unit::UnitFullCourse;
use std::env;
use dotenv::dotenv;

use rusoto_core::{Region, HttpClient};
use rusoto_credential::StaticProvider;
use rusoto_dynamodb::DynamoDbClient;

use actix_web::{web, App, HttpServer};

mod course;
mod unit;
mod class;
mod comment;
mod routes;
mod user;

#[derive(Parser, Debug)]
#[command(name = "My App", version, about = "An app that connects to MongoDB")]
struct Cli {
    #[arg(long)]
    populate: bool,
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    dotenv().ok();
    
    let cli: Cli = Cli::parse();

    let client_mongo: Client = initialize_mongo().await?;
    let client_dynamo: DynamoDbClient = initialize_dynamo().unwrap();

    if cli.populate {
        println!("Populating the database...");
        populate_database(web::Data::new(client_mongo.clone())).await?;
    }

    println!("Running server on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client_mongo.clone()))
            .app_data(web::Data::new(client_dynamo.clone()))
            .configure(unit_config) 
            .configure(courses_config)
            .configure(comment_config)
            .configure(classy_config)
            .configure(user_config)

    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

async fn initialize_mongo() -> mongodb::error::Result<Client> {
    let db_uri = env::var("MONGODB_URI").expect("Expected MONGODB_URI in env");
    let client_options = ClientOptions::parse(&db_uri).await?;
    let client = Client::with_options(client_options)?;
    println!("Connected to MongoDB!");
    Ok(client)
}

fn initialize_dynamo() -> Result<DynamoDbClient, Box<dyn std::error::Error>> {
    let dynamodb_uri: String = env::var("DYNAMODB_URI").expect("Expected DYNAMODB_URI in env");
    let access_key = env::var("ACCESS_KEY").expect("Expected ACCESS_KEY in env");
    let secret_key = env::var("SECRET_KEY").expect("Expected SECRET_KEY in env");
    let provider: StaticProvider = StaticProvider::new_minimal(access_key, secret_key);
    let client: DynamoDbClient = DynamoDbClient::new_with(
        HttpClient::new().unwrap(),
        provider,
        Region::Custom {
            name: "local".to_string(),
            endpoint: dynamodb_uri,
        },
    );
    println!("Connected to DynamoDB!");
    Ok(client)
}

pub async fn populate_database(client: web::Data<Client>) -> mongodb::error::Result<()> {
    let full_course_rust: FullCourse = FullCourse {
        name: "Mastering Rust Programming".to_string(),
        description: "A comprehensive course covering the fundamentals and advanced concepts of Rust programming.".to_string(),
        image: "rust_mastery_image.png".to_string(),
        image_banner: "rust_mastery_banner.png".to_string(),
        units: vec![
            UnitFullCourse {
                name: "Introduction to Rust".to_string(),
                order: 1,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "Getting Started with Rust".to_string(),
                        description: "An overview of Rust's history, features, and the tools you need to get started.".to_string(),
                        order: 1,
                        video: "intro_to_rust.mp4".to_string(),
                        tutor: "Alice Smith".to_string(),
                        support_material: vec!["getting_started_with_rust.pdf".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "Setting Up the Development Environment".to_string(),
                        description: "Guide to setting up Rust and the IDE for efficient development.".to_string(),
                        order: 2,
                        video: "setup_rust.mp4".to_string(),
                        tutor: "Alice Smith".to_string(),
                        support_material: vec!["rust_setup_guide.pdf".to_string()],
                    },
                ],
            },
            UnitFullCourse {
                name: "Rust Basics".to_string(),
                order: 2,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "Understanding Ownership and Borrowing".to_string(),
                        description: "A deep dive into one of Rust's core features: ownership, borrowing, and lifetimes.".to_string(),
                        order: 1,
                        video: "ownership_borrowing.mp4".to_string(),
                        tutor: "John Doe".to_string(),
                        support_material: vec!["ownership_and_borrowing_cheatsheet.pdf".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "Managing Errors with Result and Option".to_string(),
                        description: "An introduction to Rust's error-handling patterns using Result and Option types.".to_string(),
                        order: 2,
                        video: "error_handling.mp4".to_string(),
                        tutor: "John Doe".to_string(),
                        support_material: vec!["error_handling_in_rust.pdf".to_string()],
                    },
                ],
            },
            UnitFullCourse {
                name: "Advanced Rust Concepts".to_string(),
                order: 3,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "Traits and Generics".to_string(),
                        description: "Learn how to write reusable and modular code using traits and generics.".to_string(),
                        order: 1,
                        video: "traits_generics.mp4".to_string(),
                        tutor: "Alice Smith".to_string(),
                        support_material: vec!["traits_generics_examples.zip".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "Concurrency in Rust".to_string(),
                        description: "Explore Rust's memory-safe concurrency model with threads and async.".to_string(),
                        order: 2,
                        video: "concurrency_rust.mp4".to_string(),
                        tutor: "John Doe".to_string(),
                        support_material: vec!["concurrency_examples.zip".to_string()],
                    },
                ],
            },
        ],
    };

    let full_course_javascript: FullCourse = FullCourse {
        name: "Mastering JavaScript Programming".to_string(),
        description: "A comprehensive course covering JavaScript fundamentals, ES6+ features, and advanced topics like asynchronous programming.".to_string(),
        image: "javascript_mastery_image.png".to_string(),
        image_banner: "javascript_mastery_banner.png".to_string(),
        units: vec![
            UnitFullCourse {
                name: "Introduction to JavaScript".to_string(),
                order: 1,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "JavaScript Basics".to_string(),
                        description: "An introduction to JavaScript's syntax, variables, and data types.".to_string(),
                        order: 1,
                        video: "js_basics.mp4".to_string(),
                        tutor: "Emma Lee".to_string(),
                        support_material: vec!["js_basics_guide.pdf".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "Setting Up the Development Environment".to_string(),
                        description: "How to set up your environment for JavaScript development, including Node.js and IDE tips.".to_string(),
                        order: 2,
                        video: "setup_js_environment.mp4".to_string(),
                        tutor: "Emma Lee".to_string(),
                        support_material: vec!["js_setup_guide.pdf".to_string()],
                    },
                ],
            },
            UnitFullCourse {
                name: "Core JavaScript Concepts".to_string(),
                order: 2,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "Functions and Scope".to_string(),
                        description: "Understanding JavaScript functions, scope, and closures.".to_string(),
                        order: 1,
                        video: "functions_scope.mp4".to_string(),
                        tutor: "John Carter".to_string(),
                        support_material: vec!["functions_and_scope_cheatsheet.pdf".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "Object-Oriented Programming in JavaScript".to_string(),
                        description: "Exploring how JavaScript handles OOP, including constructors, prototypes, and ES6 classes.".to_string(),
                        order: 2,
                        video: "oop_js.mp4".to_string(),
                        tutor: "John Carter".to_string(),
                        support_material: vec!["oop_in_js.pdf".to_string()],
                    },
                ],
            },
            UnitFullCourse {
                name: "Advanced JavaScript".to_string(),
                order: 3,
                classes: vec![
                    ClassyReceive {
                        unit_id: None,
                        name: "Asynchronous JavaScript".to_string(),
                        description: "Deep dive into callbacks, promises, async/await, and event loops.".to_string(),
                        order: 1,
                        video: "async_js.mp4".to_string(),
                        tutor: "Emma Lee".to_string(),
                        support_material: vec!["async_js_examples.zip".to_string()],
                    },
                    ClassyReceive {
                        unit_id: None,
                        name: "JavaScript Modules and Tooling".to_string(),
                        description: "Learn about modern JavaScript modules (ES Modules, CommonJS) and tooling like npm, Webpack, and Babel.".to_string(),
                        order: 2,
                        video: "js_modules_tooling.mp4".to_string(),
                        tutor: "John Carter".to_string(),
                        support_material: vec!["modules_and_tooling_examples.zip".to_string()],
                    },
                ],
            },
        ],
    };

    let _response = create_complete_course(client.clone(), web::Json(full_course_rust)).await;
    let _response = create_complete_course(client.clone(), web::Json(full_course_javascript)).await;
    println!("MongoDB populated");
    Ok(())
}