
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
  //tide::log::start();
  let mut app = tide::new();
  app.at("/").get(|_| async { Ok("hello\n") });
  app.listen("127.0.0.1:9000").await?;
  Ok(())
}

