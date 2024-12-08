use actix_web::web;

use crate::{class::{create_class, get_classes_by_unit, get_comments_class}, comment::create_comment, course::{create_complete_course, create_course, get_available_courses, get_comments_course, get_course}, unit::{create_unit, get_units_by_course}, user::{complete_class, create_comment_user, create_table, create_user, delete_register, get_user_courses, get_users, list_tables, login_user, post_rating, register_course, update_course_status}};

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
    cfg.service(list_tables)
         .service(create_table)
        .service(create_user)
        .service(login_user)
        .service(get_users)
        .service(update_course_status)
        .service(register_course)
        .service(delete_register)
        .service(complete_class)
        .service(get_user_courses)
        .service(post_rating)
        .service(create_comment_user);
}