#[actix_web::main]
async fn main() -> std::io::Result<()> {
    backend::app::run().await
}
