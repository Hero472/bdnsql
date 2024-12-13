use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use mongodb::{bson, Collection, Cursor, Database};
use neo4rs::{query, Graph};
use rusoto_dynamodb::{AttributeValue, DeleteItemInput, DynamoDb, GetItemInput, ListTablesInput, ListTablesOutput, PutItemInput, QueryInput, ScanInput, ScanOutput, UpdateItemInput};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_dynamodb::{CreateTableInput, KeySchemaElement, AttributeDefinition, ProvisionedThroughput};
use serde::{Deserialize, Serialize};
use serde_json::json;
use core::fmt;
use std::collections::HashMap;
use maplit::hashmap;

use crate::class::Classy;
use crate::comment::{Comment, CommentReceive};
use crate::course::Course;
use crate::unit::Unit;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub pk: String,
    pub sk: String,  
    pub email: String,
    pub password: String,
    pub status: String,
    pub rating_data: String,
    pub completed_classes: Vec<String>,
    pub completion_percentage: String,
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Initiated => write!(f, "Initiated"),
            Status::InProgress(progress) => write!(f, "In Progress: {:.2}%", progress),
            Status::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseStatusUpdate {
    pub user_email: String,
    pub course_id: String,
    pub status: Status
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteClass {
    pub user_email: String,
    pub course_id: String,
    pub class_id: String
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCoursesRequest {
    pub user_email: String,
}

#[derive(Serialize)]
pub struct CourseId {
    pub course_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RatingRequest {
    user_email: String,
    course_id: String,
    rating: f32,
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
                    let completed_classes: Vec<String> = item
                        .get("completed_classes")
                        .and_then(|v| v.ss.clone())
                        .unwrap_or_default();
                    let rating_data: String = item
                        .get("rating_data")
                        .and_then(|v| v.s.clone())
                        .unwrap_or_default();
                    let completion_percentage: String = item
                        .get("completion_percentage")
                        .and_then(|v| v.n.clone())
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
                        rating_data,
                        completed_classes,
                        completion_percentage,
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

// not used lol
#[post("dynamodb/update_course_status")]
pub async fn update_course_status(
    client_dynamo: web::Data<DynamoDbClient>,
    client_mongo: web::Data<mongodb::Client>,
    course_status: web::Json<CourseStatusUpdate>,
) -> impl Responder {
    if let Err(err) = course_status.status.validate() {
        return HttpResponse::BadRequest().json(format!("Validation error: {}", err));
    }

    let pk: String = format!("user#{}", course_status.user_email.clone());
    let sk: String = format!("course#{}", course_status.course_id);

    let db: Database = client_mongo.database("local");
    //let courses_collection: Collection<Course> = db.collection::<Course>("courses");
    let units_collection: Collection<Unit> = db.collection::<Unit>("units");
    let classes_collection: Collection<Classy> = db.collection::<Classy>("classes");

    let course_oid = match bson::oid::ObjectId::parse_str(&course_status.course_id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest().json("Invalid course ID format.");
        }
    };

    // Find all units of the course
    let mut units_cursor: Cursor<Unit> = match units_collection.find(doc! { "_course_id": course_oid }).await {
        Ok(cursor) => cursor,
        Err(err) => {
            eprintln!("Failed to fetch units: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to fetch units.");
        }
    };

    let total_classes: usize;
    let mut unit_ids: Vec<ObjectId> = Vec::new();

    while let Some(result) = units_cursor.next().await {
        match result {
            Ok(unit) => {
                if let Some(id) = unit.id {
                    unit_ids.push(id);
                }
            }
            Err(err) => {
                eprintln!("Error reading unit: {:?}", err);
                return HttpResponse::InternalServerError().json("Failed to read unit.");
            }
        }
    }

    match classes_collection.count_documents(doc! { "_unit_id": { "$in": &unit_ids } }).await {
        Ok(count) => total_classes = count as usize,
        Err(err) => {
            eprintln!("Failed to count classes: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to count classes.");
        }
    }

    let get_item_input: GetItemInput = GetItemInput {
        table_name: "bdnsql".to_string(),
        key: hashmap! {
            "PK".to_string() => AttributeValue { s: Some(pk.clone()), ..Default::default() },
            "SK".to_string() => AttributeValue { s: Some(sk.clone()), ..Default::default() },
        },
        ..Default::default()
    };

    let completed_classes = match client_dynamo.get_item(get_item_input).await {
        Ok(output) => {
            if let Some(item) = output.item {
                item.get("completed_classes")
                    .and_then(|val| val.ss.as_ref()) // Get the string set
                    .map(|set| set.len()) // Get the length of the list
                    .unwrap_or(0) // Default to 0 if the key doesn't exist or is empty
            } else {
                0
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch completed classes from DynamoDB: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to fetch completed classes.");
        }
    };

    // Calculate completion percentage
    let completion_percentage: u64 = if total_classes > 0 {
        ((completed_classes as f64 / total_classes as f64) * 100.0).round() as u64
    } else {
        0
    };

    // Update DynamoDB with the new percentage
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
        s: Some(serde_json::to_string(&course_status.status).unwrap()),
        ..Default::default()
    });
    item.insert("completion_percentage".to_string(), AttributeValue {
        n: Some(completion_percentage.to_string()),
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
            println!("Course status and percentage updated successfully!");
            HttpResponse::Ok().json(format!(
                "Status for course {} updated to {:?} with {}% completion",
                course_status.course_id, course_status.status, completion_percentage
            ))
        }
        Err(err) => {
            eprintln!("Failed to update course status: {:?}", err);
            HttpResponse::InternalServerError().json(format!("Error: {:?}", err))
        }
    }

}

#[post("dynamodb/complete_class")]
pub async fn complete_class(
    client_dynamo: web::Data<DynamoDbClient>,
    client_mongo: web::Data<mongodb::Client>,
    input_data: web::Json<CompleteClass>
) -> impl Responder {
    // Validate ObjectId for course and class
    let course_oid: ObjectId = match mongodb::bson::oid::ObjectId::parse_str(&input_data.course_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid course ID format."),
    };

    let class_oid: ObjectId = match mongodb::bson::oid::ObjectId::parse_str(&input_data.class_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid class ID format."),
    };

    // MongoDB: Check if the class is part of the course
    let db: Database = client_mongo.database("local");
    let classes_collection: Collection<Classy> = db.collection::<Classy>("classes");
    match classes_collection
        .find_one(doc! { "_id": class_oid.clone() })
        .await
    {
        Ok(Some(_)) => {} // Class exists in the course, proceed
        Ok(None) => return HttpResponse::NotFound().json("Class not found in the course."),
        Err(err) => {
            eprintln!("Failed to query class in MongoDB: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to query class.");
        }
    }

    // DynamoDB: Check if user is registered in the course
    let pk: String = format!("user#{}", input_data.user_email);
    let sk: String = format!("course#{}", course_oid.to_hex());
    let mut key: HashMap<String, AttributeValue> = HashMap::new();
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

    let get_item_input: GetItemInput = GetItemInput {
        table_name: "bdnsql".to_string(),
        key: key.clone(),
        ..Default::default()
    };

    let registered: bool = match client_dynamo.get_item(get_item_input).await {
        Ok(output) => output.item.is_some(),
        Err(err) => {
            eprintln!("Failed to check registration in DynamoDB: {:?}", err);
            return HttpResponse::InternalServerError().json("Failed to check user registration.");
        }
    };

    if !registered {
        return HttpResponse::BadRequest().json("User is not registered in the course.");
    }

    // DynamoDB: Add class to the completed list
    let mut expression_attribute_values: HashMap<String, AttributeValue> = HashMap::new();
    expression_attribute_values.insert(
        ":class_id".to_string(),
        AttributeValue {
            ss: Some(vec![class_oid.to_hex()]),
            ..Default::default()
        },
    );

    let update_item_input: UpdateItemInput = UpdateItemInput {
        table_name: "bdnsql".to_string(),
        key: key.clone(),
        update_expression: Some("ADD completed_classes :class_id".to_string()),
        expression_attribute_values: Some(expression_attribute_values),
        ..Default::default()
    };

    match client_dynamo.update_item(update_item_input).await {
        Ok(_) => {
            println!("Advance in Course");

            // Fetch the updated data to calculate the new completion percentage
            let get_item_input: GetItemInput = GetItemInput {
                table_name: "bdnsql".to_string(),
                key: key.clone(),
                ..Default::default()
            };

            let course_item: Option<HashMap<String, AttributeValue>> = match client_dynamo.get_item(get_item_input).await {
                Ok(output) => output.item,
                Err(err) => {
                    eprintln!("Failed to fetch course data: {:?}", err);
                    return HttpResponse::InternalServerError().json("Failed to fetch course data.");
                }
            };

            if let Some(item) = course_item {
                let completed_classes = item
                    .get("completed_classes")
                    .and_then(|val| val.ss.as_ref())
                    .map(|set| set.len())
                    .unwrap_or(0);

                // MongoDB: Find the total number of classes for this course
                let mut units_cursor: Cursor<Unit> = match db
                    .collection::<Unit>("units")
                    .find(doc! { "_course_id": course_oid })
                    .await
                {
                    Ok(cursor) => cursor,
                    Err(err) => {
                        eprintln!("Failed to fetch units: {:?}", err);
                        return HttpResponse::InternalServerError().json("Failed to fetch units.");
                    }
                };

                let mut unit_ids: Vec<ObjectId> = Vec::new();
                while let Some(unit) = units_cursor.next().await {
                    match unit {
                        Ok(unit) => unit_ids.push(unit.id.unwrap()),
                        Err(err) => {
                            eprintln!("Failed to read unit: {:?}", err);
                            return HttpResponse::InternalServerError().json("Failed to read unit.");
                        }
                    }
                }

                let total_classes: usize = match db
                    .collection::<Classy>("classes")
                    .count_documents(doc! { "_unit_id": { "$in": &unit_ids } })
                    .await
                {
                    Ok(count) => count as usize,
                    Err(err) => {
                        eprintln!("Failed to count classes: {:?}", err);
                        return HttpResponse::InternalServerError().json("Failed to count classes.");
                    }
                };

                // Calculate new completion percentage
                let completion_percentage: u64 = if total_classes > 0 {
                    ((completed_classes as f64 / total_classes as f64) * 100.0).round() as u64
                } else {
                    0
                };

                let item_to_update: HashMap<String, AttributeValue> = HashMap::new();

                // Fetch existing completed_classes (if any), or initialize a new empty list if none
                let mut completed_classes: Vec<String> = match item_to_update.get("completed_classes") {
                    Some(value) => match &value.ss {
                        Some(classes) => classes.clone(),
                        None => Vec::new(),
                    },
                    None => Vec::new(),
                };

                let course_status = if completion_percentage >= 100 {
                    Status::Completed
                } else {
                    Status::InProgress(completion_percentage as f32)
                };

                // Add the new class ID to the list
                completed_classes.push(class_oid.to_string());
                
                // Update DynamoDB with new status and percentage
                let mut item_to_update: HashMap<String, AttributeValue> = HashMap::new();
                item_to_update.insert("PK".to_string(), AttributeValue { s: Some(pk.clone()), ..Default::default() });
                item_to_update.insert("SK".to_string(), AttributeValue { s: Some(sk.clone()), ..Default::default() });
                item_to_update.insert("status".to_string(), AttributeValue { s: Some(course_status.to_string()), ..Default::default() });
                item_to_update.insert("completed_classes".to_string(), AttributeValue { ss: Some(completed_classes.clone()), ..Default::default() });
                item_to_update.insert("completion_percentage".to_string(), AttributeValue { n: Some(completion_percentage.to_string()), ..Default::default() });
                item_to_update.insert("timestamp".to_string(), AttributeValue { s: Some(chrono::Utc::now().to_rfc3339()), ..Default::default() });

                let put_item_input = PutItemInput {
                    table_name: "bdnsql".to_string(),
                    item: item_to_update,
                    ..Default::default()
                };

                match client_dynamo.put_item(put_item_input).await {
                    Ok(_) => {
                        HttpResponse::Ok().json(format!(
                            "Class completed and course updated to InProgress with {}% completion.",
                            completion_percentage
                        ))
                    }
                    Err(err) => {
                        eprintln!("Failed to update course status: {:?}", err);
                        HttpResponse::InternalServerError().json("Failed to update course status.")
                    }
                }
            } else {
                HttpResponse::InternalServerError().json("Failed to retrieve course item.")
            }
        }
        Err(err) => {
            eprintln!("Failed to update completed classes in DynamoDB: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to mark class as completed.")
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
    item.insert("completion_percentage".to_string(), AttributeValue {
        n: Some(0.to_string()),
        ..Default::default()
    });
    item.insert("timestamp".to_string(), AttributeValue {
        s: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    });
    item.insert("rating_data".to_string(), AttributeValue {
        s: Some("".to_string()),
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

#[get("dynamodb/get_user_courses")]
pub async fn get_user_courses(
    client_dynamo: web::Data<DynamoDbClient>,
    input_data: web::Json<UserCoursesRequest>,
) -> impl Responder {
    let pk: String = format!("user#{}", input_data.user_email);

    // Construct the QueryInput to fetch all courses associated with the user
    let query_input = QueryInput {
        table_name: "bdnsql".to_string(),
        key_condition_expression: Some("PK = :pk and begins_with(SK, :sk_prefix)".to_string()),
        expression_attribute_values: Some(hashmap! {
            ":pk".to_string() => AttributeValue { s: Some(pk.clone()), ..Default::default() },
            ":sk_prefix".to_string() => AttributeValue { s: Some("course#".to_string()), ..Default::default() },
        }),
        ..Default::default()
    };

    match client_dynamo.query(query_input).await {
        Ok(output) => {
            // Process the query results and collect course IDs
            if let Some(items) = output.items {
                let course_ids: Vec<CourseId> = items.into_iter().filter_map(|item| {
                    item.get("SK").and_then(|val| val.s.clone()).map(|sk| {
                        CourseId {
                            course_id: sk.strip_prefix("course#").unwrap_or_default().to_string(),
                        }
                    })
                }).collect();

                // Return the list of course IDs
                HttpResponse::Ok().json(course_ids)
            } else {
                HttpResponse::NotFound().json("No courses found for the user.")
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch user courses from DynamoDB: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to fetch user courses.")
        }
    }
}

#[post("neo4j/rating")]
pub async fn post_rating(
    neo4j_graph: web::Data<Graph>,
    client_mongo: web::Data<mongodb::Client>,
    input_data: web::Json<RatingRequest>,
) -> impl Responder {
    // Validate the rating
    if input_data.rating < 1.0 || input_data.rating > 5.0 {
        return HttpResponse::BadRequest().json("Rating must be between 1 and 5");
    }

    // MongoDB: Get the course collection
    let db: Database = client_mongo.database("local");
    let courses_collection: Collection<Course> = db.collection("courses");

    // Parse the course ID
    let course_oid: ObjectId = match ObjectId::parse_str(&input_data.course_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid course ID format."),
    };

    // Find the course and update its rating
    match courses_collection.find_one(doc! { "_id": course_oid }).await {
        Ok(Some(course)) => {
            let current_total_rates: f32 = course.total_rates as f32;
            let current_rating: f32 = course.rating.unwrap_or(0.0);
            let new_total_rates: f32 = current_total_rates + 1.0;
            let new_rating: f32 =
                (current_rating * current_total_rates + input_data.rating) / new_total_rates;

            // Update the course in MongoDB
            let update: Document = doc! {
                "$set": {
                    "rating": new_rating,
                },
                "$inc": {
                    "total_rates": 1,
                },
            };



            match courses_collection.update_one( doc! { "_id": course_oid }, update).await {
                Ok(_) => {
                    // Register the rating in Neo4j
                    let neo4j_query = query(
                        "MERGE (u:User {email: $email}) \
                         MERGE (c:Course {id: $course_id}) \
                         CREATE (u)-[:RATED {rating: $rating, timestamp: $timestamp}]->(c)")
                        .param("email", input_data.user_email.clone())
                        .param("course_id", input_data.course_id.clone())
                        .param("rating", input_data.rating)
                        .param("timestamp", chrono::Utc::now().to_rfc3339());
        
                    if let Err(err) = neo4j_graph.run(neo4j_query).await {
                        eprintln!("Neo4j error during rating registration: {}", err);
                        return HttpResponse::InternalServerError().json("Failed to register rating in Neo4j.");
                    }
        
                    HttpResponse::Ok().json("Rating submitted successfully.")
                },
                Err(err) => {
                    eprintln!("Database error during rating update: {}", err);
                    HttpResponse::InternalServerError().json("Failed to update course rating.")
                }
            }

        }
        Ok(None) => HttpResponse::NotFound().json("Course not found."),
        Err(err) => {
            eprintln!("MongoDB error during fetch: {}", err);
            HttpResponse::InternalServerError().json("Failed to fetch the course.")
        }
    }
}

#[post("dynamodb/rating")]
pub async fn post_rating_neo4j(
    client_dynamo: web::Data<DynamoDbClient>,
    client_mongo: web::Data<mongodb::Client>,
    input_data: web::Json<RatingRequest>,
) -> impl Responder {
    // Validate the rating
    if input_data.rating < 1.0 || input_data.rating > 5.0 {
        return HttpResponse::BadRequest().json("Rating must be between 1 and 5");
    }

    // MongoDB: Get the course collection
    let db: Database = client_mongo.database("local");
    let courses_collection: Collection<Course> = db.collection("courses");

    // Parse the course ID
    let course_oid: ObjectId = match ObjectId::parse_str(&input_data.course_id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json("Invalid course ID format."),
    };

    // Find the course and update its rating
    match courses_collection.find_one(doc! { "_id": course_oid }).await {
        Ok(Some(course)) => {
            let current_total_rates: f32 = course.total_rates as f32;
            let current_rating: f32 = course.rating.unwrap_or(0.0);
            let new_total_rates: f32 = current_total_rates + 1.0;
            let new_rating: f32 =
                (current_rating * current_total_rates + input_data.rating) / new_total_rates;

            // Update the course in MongoDB
            let update: Document = doc! {
                "$set": {
                    "rating": new_rating,
                },
                "$inc": {
                    "total_rates": 1,
                },
            };

            if let Err(err) = courses_collection
                .update_one(doc! { "_id": course_oid }, update)
                .await
            {
                eprintln!("MongoDB error during course update: {}", err);
                return HttpResponse::InternalServerError().json("Failed to update the course rating.");
            }

            // DynamoDB: Save the rating under the user's data
            let rating_entry: serde_json::Value = json!({
                "course_id": input_data.course_id,
                "rating": input_data.rating,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            let mut item: HashMap<String, AttributeValue> = HashMap::new();

            let user_pk: String = format!("user#{}", input_data.user_email);
            if user_pk.is_empty() {
                eprintln!("Error: User email is missing.");
                return HttpResponse::BadRequest().json("User email is required.");
            }
            item.insert(
                "PK".to_string(),
                AttributeValue {
                    s: Some(user_pk.clone()),
                    ..Default::default()
                },
            );

            // Sort Key
            let sk: String = format!("course#{}", input_data.course_id);
            if sk.is_empty() {
                eprintln!("Error: Course ID is missing.");
                return HttpResponse::BadRequest().json("Course ID is required.");
            }
            item.insert(
                "SK".to_string(),
                AttributeValue {
                    s: Some(sk.clone()),
                    ..Default::default()
                },
            );

            // Add rating data
            let rating_data: String = rating_entry.to_string();
            println!("{rating_data}");
            if rating_data.is_empty() {
                eprintln!("Error: Rating data is empty.");
                return HttpResponse::BadRequest().json("Rating data is required.");
            }
            item.insert(
                "rating_data".to_string(),
                AttributeValue {
                    s: Some(rating_data.to_string()),
                    ..Default::default()
                },
            );

            // Execute the PutItem operation
            let dynamo_put = client_dynamo
                .put_item(PutItemInput {
                    table_name: "bdnsql".to_string(),
                    item,
                    ..Default::default()
                })
                .await;

            match dynamo_put {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Rating submitted successfully and saved to user data."
                })),
                Err(err) => {
                    eprintln!("DynamoDB error during submission: {}", err);
                    HttpResponse::InternalServerError().json("Failed to submit the rating to user data.")
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json("Course not found."),
        Err(err) => {
            eprintln!("MongoDB error during fetch: {}", err);
            HttpResponse::InternalServerError().json("Failed to fetch the course.")
        }
    }
}


#[post("dynamodb/create_comment")]
pub async fn create_comment_user(
    mongo_client: web::Data<mongodb::Client>,
    dynamo_client: web::Data<DynamoDbClient>,
    new_comment: web::Json<CommentReceive>,
) -> impl Responder {
    let db: Database = mongo_client.database("local");
    let collection: Collection<Comment> = db.collection("comments");

    // MongoDB: Create the new comment
    let new_comment_data: Comment = Comment {
        id: None,
        author: new_comment.author.clone(),
        date: Utc::now(),
        title: new_comment.title.clone(),
        detail: new_comment.detail.clone(),
        likes: 0,
        dislikes: 0,
        reference_id: new_comment.reference_id.clone(),
        reference_type: new_comment.reference_type.clone(),
    };

    // Insert the comment into MongoDB
    match collection.insert_one(&new_comment_data).await {
        Ok(insert_result) => {
            // Extract the inserted ID
            let inserted_id: ObjectId = insert_result.inserted_id.as_object_id().unwrap();

            // DynamoDB: Prepare the item
            let mut item: HashMap<String, AttributeValue> = HashMap::new();

            // Partition Key (PK): "user#<author>"
            let pk: String = format!(
                "user#{}",
                new_comment.author
            );
            item.insert(
                "PK".to_string(),
                AttributeValue {
                    s: Some(pk),
                    ..Default::default()
                },
            );

            // Sort Key (SK): "comment#<comment_id>"
            let sk: String = format!("comment#{}", inserted_id.to_hex());
            item.insert(
                "SK".to_string(),
                AttributeValue {
                    s: Some(sk),
                    ..Default::default()
                },
            );

            // Additional attributes
            item.insert(
                "author".to_string(),
                AttributeValue {
                    s: Some(new_comment.author.clone()),
                    ..Default::default()
                },
            );
            item.insert(
                "title".to_string(),
                AttributeValue {
                    s: Some(new_comment.title.clone()),
                    ..Default::default()
                },
            );
            item.insert(
                "detail".to_string(),
                AttributeValue {
                    s: Some(new_comment.detail.clone()),
                    ..Default::default()
                },
            );
            item.insert(
                "date".to_string(),
                AttributeValue {
                    s: Some(Utc::now().to_rfc3339()),
                    ..Default::default()
                },
            );
            item.insert(
                "type".to_string(),
                AttributeValue {
                    s: Some(new_comment.reference_type.to_string()),
                    ..Default::default()
                },
            );

            // Insert the item into DynamoDB
            let dynamo_put = dynamo_client
                .put_item(PutItemInput {
                    table_name: "bdnsql".to_string(),
                    item,
                    ..Default::default()
                })
                .await;

            match dynamo_put {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Rating submitted successfully and saved to user data."
                })),
                Err(err) => {
                    eprintln!("DynamoDB error during submission: {}", err);
                    HttpResponse::InternalServerError().json("Failed to submit the rating to user data.")
                }
            }
        }
        Err(err) => {
            eprintln!("MongoDB error: {}", err);
            HttpResponse::InternalServerError().body("Failed to save the comment in MongoDB")
        }
    }
}

#[post("neo4j/create_comment")]
pub async fn create_comment_user_neo4j(
    mongo_client: web::Data<mongodb::Client>,
    neo4j_graph: web::Data<Graph>,
    new_comment: web::Json<CommentReceive>,
) -> impl Responder {
    let db: Database = mongo_client.database("local");
    let collection: Collection<Comment> = db.collection("comments");

    // MongoDB: Create the new comment
    let new_comment_data: Comment = Comment {
        id: None,
        author: new_comment.author.clone(),
        date: Utc::now(),
        title: new_comment.title.clone(),
        detail: new_comment.detail.clone(),
        likes: 0,
        dislikes: 0,
        reference_id: new_comment.reference_id.clone(),
        reference_type: new_comment.reference_type.clone(),
    };

    // Insert the comment into MongoDB
    match collection.insert_one(&new_comment_data).await {
        Ok(insert_result) => {
            // Extract the inserted ID
            let inserted_id: ObjectId = insert_result.inserted_id.as_object_id().unwrap();

            let mut params = HashMap::new();
            params.insert("author", new_comment.author.clone().into());
            params.insert("comment_id", inserted_id.to_hex().into());
            params.insert("title", new_comment.title.clone().into());
            params.insert("detail", new_comment.detail.clone().into());
            params.insert("date", Utc::now().to_rfc3339().into());
            params.insert("reference_id", new_comment.reference_id.clone().into());
            params.insert("reference_type", new_comment.reference_type.clone().into());

            let cypher_query = "
                MATCH (user:User {email: $author})
                CREATE (comment:Comment {
                    id: $comment_id,
                    title: $title,
                    detail: $detail,
                    date: $date,
                    reference_id: $reference_id,
                    reference_type: $reference_type,
                    likes: 0,
                    dislikes: 0
                })
                CREATE (user)-[:POSTED]->(comment)
            ";

            let graph = neo4j_graph.get_ref();
            match graph
                .execute(
                    query(
                        "CREATE (c:Comment {
                            comment_id: $comment_id, 
                            author: $author, 
                            title: $title, 
                            detail: $detail, 
                            date: $date, 
                            reference_id: $reference_id, 
                            reference_type: $reference_type
                        })",
                    )
                    .params(params),
                )
                .await
            {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Comment successfully created in MongoDB and Neo4j"
                })),
                Err(err) => {
                    eprintln!("Neo4j error: {}", err);
                    HttpResponse::InternalServerError().body("Failed to save the comment in Neo4j")
                }
            }
        }
        Err(err) => {
            eprintln!("MongoDB error: {}", err);
            HttpResponse::InternalServerError().body("Failed to save the comment in MongoDB")
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