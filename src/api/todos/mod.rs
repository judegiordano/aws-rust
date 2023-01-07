use actix_web::web::{self, ServiceConfig};

pub mod controller;

pub fn router(cfg: &mut ServiceConfig) {
    cfg.route("", web::get().to(controller::list_todos));
    cfg.route("/{_id}", web::get().to(controller::read_todo));
    cfg.route("", web::post().to(controller::create_todo));
    cfg.route("/complete", web::put().to(controller::complete_todo));
}
