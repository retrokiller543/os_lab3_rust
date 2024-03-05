use actix_web::{web, App, HttpServer};
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/wasm", "./public").show_files_listing())
        // You can also configure routes to serve your `diskfile.bin` and logs from the specific directory
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}