use actix_web::web::{scope, ServiceConfig};

pub mod dev;
pub mod todos;
pub mod users;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/developer").configure(dev::router));
    cfg.service(scope("/todos").configure(todos::router));
    cfg.service(scope("/users").configure(users::router));
}
