// https://github.com/actix/examples/blob/master/hello-world/src/main.rs

use actix_web::{web, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| App::new().service(web::resource("/").to(|| async { "hello\n" })))
        .bind("0.0.0.0:9000")?
        .run()
        .await
}
