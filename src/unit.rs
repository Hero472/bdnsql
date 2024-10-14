use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::{bson::{doc, oid::ObjectId}, Collection};
use serde::{Deserialize, Serialize};

use crate::course::Course;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Unit {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "_course_id", skip_serializing_if = "Option::is_none")]
    course_id: Option<ObjectId>,
    name: String,
    order: usize
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitReceive {
    course_id: Option<ObjectId>,
    name: String,
    order: usize
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitSend {
    unit_id: Option<ObjectId>,
    name: String,
    order: usize
}

pub async fn create_unit(client: web::Data<mongodb::Client>, new_unit: web::Json<UnitReceive>) -> impl Responder {
    let db: mongodb::Database = client.database("local");
    let collection_units: Collection<Unit> = db.collection("units");
    let collection_courses: Collection<Course> = db.collection("courses");

    let new_unit_data: Unit = Unit {
        id: None,
        course_id: new_unit.course_id,
        name: new_unit.name.clone(),
        order: new_unit.order
    };

    match collection_units.insert_one(new_unit_data.clone()).await {
        Ok(insert_result) => {
            let unit_id: ObjectId = insert_result.inserted_id.as_object_id().unwrap();

            let course_id: Option<ObjectId> = new_unit_data.course_id.clone();
            let course_id_str: String = course_id.map(|id| id.to_hex()).unwrap();

            let course_id: ObjectId = match ObjectId::parse_str(course_id_str) {
                Ok(id) => id,
                Err(_) => return HttpResponse::BadRequest().body("Invalid course ID format."),
            };

            match collection_courses.find_one(doc! { "_id": course_id }).await {
                Ok(Some(mut course)) => {
                    course.units.push(unit_id.clone());
                    course.units.sort();
                    match collection_courses.update_one(
                        doc! { "_id": course_id },
                        doc! { "$set": { "units": course.units } }
                    ).await {
                        Ok(_) => HttpResponse::Ok().json(insert_result.inserted_id),
                        Err(e) => HttpResponse::InternalServerError().body(format!("Error updating course: {}", e)),
                    }
                },
                Ok(None) => HttpResponse::NotFound().body("Course not found."),
                Err(e) => HttpResponse::InternalServerError().body(format!("Error retrieving course: {}", e)),
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Error inserting unit: {}", e)),
    }
}

pub async fn get_units_by_course(client: web::Data<mongodb::Client>, course_id: web::Path<String>) -> impl Responder {
    let db: mongodb::Database = client.database("local");
    let collection: Collection<Unit> = db.collection("units");

    let course_id: ObjectId = match ObjectId::parse_str(&*course_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Course ID format")
    };

    let mut cursor: mongodb::Cursor<Unit> = match collection.find(doc! {"_course_id": course_id}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut units: Vec<UnitSend> = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(unit_doc) => {
                
                let unit_send: UnitSend = UnitSend {
                    unit_id: unit_doc.id,
                    name: unit_doc.name,
                    order: unit_doc.order,
                };

                units.push(unit_send)
            },
            Err(e) => eprintln!("Error processing document: {:?}", e),
        }
    }
    
    units.sort_by(|a: &UnitSend, b: &UnitSend| a.order.cmp(&b.order));

    HttpResponse::Ok().json(units)
}