use actix_web::web::{self, ServiceConfig};

pub mod controller;

pub fn router(cfg: &mut ServiceConfig) {
    cfg.route("/users", web::get().to(controller::list_users));
    cfg.route("/users", web::post().to(controller::create_user));
    cfg.route("/users/{id}", web::get().to(controller::read_by_id));
    cfg.route("/users/{id}", web::delete().to(controller::delete_by_id));
}
