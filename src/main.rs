use std::path::Path;

use rocket::{launch, fs::{FileServer, relative, NamedFile}, get, routes};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("web/public")))
        .mount("/", routes![lobby])
}

#[get("/lobby")]
async fn lobby() -> Option<NamedFile> {
    NamedFile::open(Path::new("web/protected/lobby.html")).await.ok()
}
