use actix_web::web;

use crate::{class::{create_class, get_classes_by_unit, get_comments_class}, comment::create_comment, course::{create_complete_course, create_course, get_available_courses, get_comments_course, get_course}, unit::{create_unit, get_units_by_course}, user::{complete_class, create_comment_user, create_comment_user_neo4j, create_table, create_user, delete_register, get_user_courses, get_users, list_tables, login_user, post_rating, post_rating_neo4j, register_course, update_course_status}};

pub fn unit_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/units")
            .route(web::post().to(create_unit))
    )
    .service(
        web::resource("/units/{course_id}")
            .route(web::get().to(get_units_by_course))
    );
}

pub fn courses_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/courses")
            .route(web::post().to(create_course))
            .route(web::get().to(get_available_courses))
    )
    .service(
        web::resource("/courses/{course_id}")
            .route(web::get().to(get_course))
    )
    .service(
        web::resource("/courses/comments/{course_id}")
            .route(web::get().to(get_comments_course))
    ).service(
        web::resource("/full-course")
        .route(web::post().to(create_complete_course))
    );
}

pub fn comment_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/comments")
            .route(web::post().to(create_comment))
    );
}

pub fn classy_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/classes")
            .route(web::post().to(create_class)) 
    )
    .service(
        web::resource("/classes/unit/{unit_id}")
            .route(web::get().to(get_classes_by_unit))
    )
    .service(
        web::resource("/classes/comments/{class_id}")
            .route(web::get().to(get_comments_class))
    );
}

pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/dynamodb/tables")
            .route(web::get().to(list_tables)),
    )
    .service(
        web::resource("/dynamodb/create_table")
            .route(web::post().to(create_table)),
    )
    .service(
        web::resource("/dynamodb/register")
            .route(web::post().to(create_user)),
    )
    .service(
        web::resource("dynamodb/login")
            .route(web::post().to(login_user)),
    )
    .service(
        web::resource("/dynamodb/get_users")
            .route(web::get().to(get_users)),
    )
    .service(
        web::resource("dynamodb/update_course_status")
            .route(web::post().to(update_course_status)),
    )
    .service(
        web::resource("dynamodb/register_course")
            .route(web::post().to(register_course)),
    )
    .service(
        web::resource("dynamodb/delete_register")
            .route(web::delete().to(delete_register)),
    )
    .service(
        web::resource("dynamodb/complete_class")
            .route(web::post().to(complete_class)),
    )
    .service(
        web::resource("dynamodb/get_user_courses")
            .route(web::get().to(get_user_courses)),
    )
    .service(
        web::resource("dynamodb/rating")
            .route(web::post().to(post_rating)),
    )
    .service(
        web::resource("dynamodb/create_comment")
            .route(web::post().to(create_comment_user)),
    )
    .service(
        web::resource("/rating/neo4j")
            .route(web::post().to(post_rating_neo4j)),
    )
    .service(
        web::resource("neo4j/create_comment")
            .route(web::post().to(create_comment_user_neo4j)),
    );
}