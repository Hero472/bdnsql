use actix_web::{delete, get, post, web, HttpResponse, Responder};
use mongodb::bson::doc;
use mongodb::{bson, Collection, Database};
use rusoto_dynamodb::{AttributeValue, DeleteItemInput, DynamoDb, GetItemInput, ListTablesInput, ListTablesOutput, PutItemInput, ScanInput, ScanOutput};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_dynamodb::{CreateTableInput, KeySchemaElement, AttributeDefinition, ProvisionedThroughput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::course::Course;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub pk: String,
    pub sk: String,  
    pub email: String,
    pub password: String,
    pub status: String,
    pub timestamp: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Status {
    Initiated,
    InProgress(f32),
    Completed
}

impl Status {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Status::InProgress(progress) => {
                if *progress < 0.0 || *progress > 100.0 {
                    return Err("Progress must be between 0 and 100.".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseStatusUpdate {
    pub user_email: String,
    pub course_id: String,
    pub status: Status
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseStatusRegister {
    pub user_email: String,
    pub course_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRegisterRequest {
    pub user_email: String,
    pub course_id: String,
}

#[get("dynamodb/tables")]
pub async fn list_tables(client_dynamo: web::Data<DynamoDbClient>) -> impl Responder {
    let input: ListTablesInput = ListTablesInput {limit:Some(10), exclusive_start_table_name: None };
    match client_dynamo.list_tables(input).await {
        Ok(ListTablesOutput { table_names, .. }) => {
            HttpResponse::Ok().json(table_names.unwrap_or_default())
        }
        Err(err) => {
            eprintln!("Failed to list tables: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to list tables")
        }
    }
}

#[post("dynamodb/create_table")]
pub async fn create_table(client_dynamo: web::Data<DynamoDbClient>) -> impl Responder {
    let input: CreateTableInput = CreateTableInput {
        table_name: "bdnsql".to_string(),
        key_schema: vec![
            KeySchemaElement {
                attribute_name: "PK".to_string(),
                key_type: "HASH".to_string()
            },
            KeySchemaElement {
                attribute_name: "SK".to_string(),
                key_type: "RANGE".to_string()
            },
        ],
        attribute_definitions: vec![
            AttributeDefinition {
                attribute_name: "PK".to_string(),
                attribute_type: "S".to_string()
            },
            AttributeDefinition {
                attribute_name: "SK".to_string(),
                attribute_type: "S".to_string()
            },
        ],
        provisioned_throughput: Some(ProvisionedThroughput {
            read_capacity_units: 5,
            write_capacity_units: 5
        }),

        ..Default::default()
    };

    match client_dynamo.create_table(input).await {
        Ok(_) => {
            println!("Table created successfully!");
            HttpResponse::Ok().json("Table created successfully")
        },
        Err(err) => {
            eprintln!("Failed to create table: {:?}", err);
            HttpResponse::InternalServerError().json(format!("Failed to create table: {:?}", err))
        },
    }
}

#[get("/dynamodb/get_users")]
pub async fn get_users(client_dynamo: web::Data<DynamoDbClient>) -> impl Responder {
    let scan_input: ScanInput = ScanInput {
        table_name: "bdnsql".to_string(),
        ..Default::default()
    };

    // Perform the scan operation
    match client_dynamo.scan(scan_input).await {
        Ok(ScanOutput { items, .. }) => {
            let mut users: Vec<User> = Vec::new();

            if let Some(items) = items {
                for item in items {
                    let pk: String = item
                        .get("PK")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let sk: String = item
                        .get("SK")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let email: String = item
                        .get("email")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let password: String = item
                        .get("password")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let status: String = item
                        .get("status")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let timestamp: String = item
                        .get("timestamp")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();

                    let user: User = User {
                        pk,
                        sk,
                        email,
                        password,
                        status,
                        timestamp
                    };

                    users.push(user);
                }
            }

            HttpResponse::Ok().json(users)
        },
        Err(err) => {
            eprintln!("Failed to get users: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}

#[post("dynamodb/register")]
pub async fn create_user(client_dynamo: web::Data<DynamoDbClient>, user: web::Json<User>) -> impl Responder {
    let email: String = user.email.clone();
    let password: String = user.password.clone();

    let pk: String = format!("user#{}", email);
    let sk: String = "USER".to_string();

    let mut item: HashMap<String, AttributeValue> = std::collections::HashMap::new();

    item.insert(
        "PK".to_string(),
        AttributeValue {
            s: Some(pk.clone()),
            ..Default::default()
        },
    );
    item.insert(
        "SK".to_string(),
        AttributeValue {
            s: Some(sk.clone()),
            ..Default::default()
        },
    );

    item.insert(
        "email".to_string(),
        AttributeValue {
            s: Some(email.clone()),
            ..Default::default()
        },
    );
    item.insert(
        "password".to_string(),
        AttributeValue {
            s: Some(password.clone()),
            ..Default::default()
        },
    );

    let put_item_input: PutItemInput = PutItemInput {
        table_name: "bdnsql".to_string(),
        item,
        ..Default::default()
    };

    match client_dynamo.put_item(put_item_input).await {
        Ok(_) => {
            println!("User registered successfully!");
            HttpResponse::Ok().json(format!("User {} registered", email))
        }
        Err(err) => {
            println!("Failed to register user: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}

#[get("dynamodb/login")]
pub async fn login_user(client_dynamo: web::Data<DynamoDbClient>, user: web::Json<User>) -> impl Responder {
    let email: String = user.email.clone();
    let password: String = user.password.clone();

    let pk: String = format!("user#{}", email);
    let sk: String = "USER".to_string();

    let key = HashMap::from([
        ("PK".to_string(), AttributeValue {
            s: Some(pk.clone()),
            ..Default::default()
        }),
        ("SK".to_string(), AttributeValue {
            s: Some(sk.clone()),
            ..Default::default()
        })
    ]);

    let get_item_input: GetItemInput = GetItemInput {
        table_name: "bdnsql".to_string(),
        key,
        ..Default::default()
    };

    match client_dynamo.get_item(get_item_input).await {
        Ok(response) => {
            if let Some(item) = response.item {
                if let Some(stored_password) = item.get("password") {
                    if let Some(stored_password_value) = &stored_password.s {
                        if stored_password_value == &password {
                            return HttpResponse::Ok().json(format!("User {} logged in successfully", email));
                        } else {
                            return HttpResponse::Unauthorized().json("Invalid password");
                        }
                    }
                }
            }
            HttpResponse::NotFound().json("User not found")
        },
        Err(err) => {
            println!("Failed to query user: {}", err);
            HttpResponse::InternalServerError().json("Error during login")
        }
    }
}

// No funciona aun
#[post("dynamodb/update_course_status")]
pub async fn update_course_status(
    client_dynamo: web::Data<DynamoDbClient>,
    course_status: web::Json<CourseStatusUpdate>,
) -> impl Responder {
    if let Err(err) = course_status.status.validate() {
        return HttpResponse::BadRequest().json(format!("Validation error: {}", err));
    }

    let pk: String = format!("user#{}", course_status.user_email.clone());
    let sk: String = format!("course#{}", course_status.course_id);

    let status_value: String = serde_json::to_string(&course_status.status)
        .map_err(|err| format!("Failed to serialize status: {:?}", err))
        .unwrap();

    let mut item: HashMap<String, AttributeValue> = HashMap::new();
    item.insert("PK".to_string(), AttributeValue {
        s: Some(pk.clone()),
        ..Default::default()
    });
    item.insert("SK".to_string(), AttributeValue {
        s: Some(sk.clone()),
        ..Default::default()
    });
    item.insert("status".to_string(), AttributeValue {
        s: Some(status_value),
        ..Default::default()
    });
    item.insert("timestamp".to_string(), AttributeValue {
        s: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    });

    let put_item_input: PutItemInput = PutItemInput {
        table_name: "bdnsql".to_string(),
        item,
        ..Default::default()
    };

    match client_dynamo.put_item(put_item_input).await {
        Ok(_) => {
            println!("Course status updated successfully!");
            HttpResponse::Ok().json(format!(
                "Status for course {} updated to {:?}",
                course_status.course_id, course_status.status
            ))
        }
        Err(err) => {
            eprintln!("Failed to update course status: {:?}", err);
            HttpResponse::InternalServerError().json(format!("Error: {:?}", err))
        }
    }
}

#[post("dynamodb/register_course")]
pub async fn register_course(
    client_dynamo: web::Data<DynamoDbClient>,
    client_mongo: web::Data<mongodb::Client>,
    register_request: web::Json<CourseStatusRegister>,
) -> impl Responder {
    let course_status: CourseStatusRegister = register_request.into_inner();
    let course_id: String = course_status.course_id.clone();

    let pk: String = format!("user#{}", course_status.user_email.clone());
    let sk: String = format!("course#{}", course_id);

    let get_item_input: GetItemInput = GetItemInput {
        table_name: "bdnsql".to_string(),
        key: {
            let mut key: HashMap<String, AttributeValue> = HashMap::new();
            key.insert("PK".to_string(), AttributeValue {
                s: Some(pk.clone()),
                ..Default::default()
            });
            key.insert("SK".to_string(), AttributeValue {
                s: Some(sk.clone()),
                ..Default::default()
            });
            key
        },
        ..Default::default()
    };

    match client_dynamo.get_item(get_item_input).await {
        Ok(get_item_output) => {
            if get_item_output.item.is_some() {
                return HttpResponse::BadRequest()
                    .json("User is already registered for this course.");
            }
        }
        Err(err) => {
            eprintln!("Failed to check registration in DynamoDB: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(format!("DynamoDB error: {:?}", err));
        }
    }


    let status_value: String = serde_json::to_string(&Status::Initiated)
        .map_err(|err| format!("Failed to serialize status: {:?}", err))
        .unwrap();

    let mut item: HashMap<String, AttributeValue> = HashMap::new();

    item.insert("PK".to_string(), AttributeValue {
        s: Some(pk.clone()),
        ..Default::default()
    });
    item.insert("SK".to_string(), AttributeValue {
        s: Some(sk.clone()),
        ..Default::default()
    });
    item.insert("status".to_string(), AttributeValue {
        s: Some(status_value),
        ..Default::default()
    });
    item.insert("timestamp".to_string(), AttributeValue {
        s: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    });

    let put_item_input: PutItemInput = PutItemInput {
        table_name: "bdnsql".to_string(),
        item,
        ..Default::default()
    };

    if let Err(err) = client_dynamo.put_item(put_item_input).await {
        eprintln!("Failed to register course in DynamoDB: {:?}", err);
        return HttpResponse::InternalServerError().json(format!("DynamoDB error: {:?}", err));
    }

    let db: Database = client_mongo.database("local");
    let courses_collection: Collection<Course> = db.collection("courses");

    let course_oid: bson::oid::ObjectId = match bson::oid::ObjectId::parse_str(&course_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid course ID format."),
    };

    let update_result = courses_collection
        .update_one(
            doc! { "_id": course_oid },
            doc! { "$inc": { "inscribed": 1 } }
        )
        .await;

    match update_result {
        Ok(update_result) => {
            if update_result.matched_count == 0 {
                println!("Course Not found in MongoDB");
                return HttpResponse::NotFound().json("Course not found in MongoDB.");
            }
            println!("Course inscribed count updated successfully!");
            HttpResponse::Ok().json("User registered to course successfully!")
        }
        Err(err) => {
            eprintln!("Failed to update course inscribed count in MongoDB: {:?}", err);
            HttpResponse::InternalServerError().json(format!("MongoDB error: {:?}", err))
        }
    }
}

#[delete("dynamodb/delete_register")]
pub async fn delete_register(
    client_dynamo: web::Data<DynamoDbClient>,
    delete_request: web::Json<DeleteRegisterRequest>,
) -> impl Responder {
    let pk: String = format!("user#{}", delete_request.user_email);
    let sk: String = format!("course#{}", delete_request.course_id);

    let delete_item_input: DeleteItemInput = DeleteItemInput {
        table_name: "bdnsql".to_string(),
        key: {
            let mut key = HashMap::new();
            key.insert(
                "PK".to_string(),
                AttributeValue {
                    s: Some(pk.clone()),
                    ..Default::default()
                },
            );
            key.insert(
                "SK".to_string(),
                AttributeValue {
                    s: Some(sk.clone()),
                    ..Default::default()
                },
            );
            key
        },
        ..Default::default()
    };

    match client_dynamo.delete_item(delete_item_input).await {
        Ok(_) => {
            println!("Register deleted successfully for PK: {}, SK: {}", pk, sk);
            HttpResponse::Ok().json("Register deleted successfully")
        }
        Err(err) => {
            eprintln!("Failed to delete register: {:?}", err);
            HttpResponse::InternalServerError().json(format!("Error: {:?}", err))
        }
    }
}