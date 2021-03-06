#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use std::io;
use std::io::Error;
use std::fs;
use std::fs::ReadDir;
use std::env;
use std::path::{Path, PathBuf};

use rocket::Data;
use rocket::Request;
use rocket::Response;
use rocket::http::Status;
use rocket::response::NamedFile;
use rocket::response::Responder;
use rocket_contrib::Json;

enum RetrievedData {
  Certification(NamedFile),
  Index(ReadDir),
}
use RetrievedData::*;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct DirEntry {
  name: String,
  #[serde(rename = "type")]
  type_: String,
}

impl<'r> Responder<'r> for RetrievedData {

  fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
    match self {
      Certification(file) => file.respond_to(request),
      Index(dir) => {
        let mut entries = Vec::new();
        for entry in dir {
          match entry {
            Ok(e) => {
              let type_;
              if e.path().is_file() {
                type_ = "file"
              } else if e.path().is_dir() {
                type_ = "dir"
              } else {
                type_ = "unknown"
              }
              match e.path().to_str() {
                Some(p) => {
                  let name;
                  if p.starts_with("./") {
                    name = &p[2..];
                  } else {
                    name = p;
                  };
                  entries.push(DirEntry {
                    name: name.to_string(),
                    type_: type_.to_string(),
                  });
                },
                None => ()
              }
            },
            Err(_) => ()
          }
        };
        Json(entries).respond_to(request)
      }
    }
  }
}

#[get("/")]
fn root() -> Result<RetrievedData, Error> {
  fs::read_dir(".").map(Index)
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Result<RetrievedData, String> {
    if file.is_dir() {
      fs::read_dir(file).map(Index).map_err(|_| "Could not open".to_string())
    } else {
      NamedFile::open(file).map(Certification).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
        "Does not exist".to_string()
      } else {
        "Other error".to_string()
      }
    })
    }
}

#[put("/<file..>", data = "<body>")]
fn puts(file: PathBuf, body: Data) -> io::Result<String> {
    fs::create_dir_all(file.clone().parent().unwrap())?;
    if file.exists() {
        // TODO check if contents are identical, if so, return 200
        // TODO rocket turns this into a 404, find out how to return
        // a better 4xx error message
        Err(io::Error::new(io::ErrorKind::Other, "Already exists"))
    } else {
        body.stream_to_file(file).map(|_| "OK".to_string())
    }
}

fn main() {
    // TODO error responses are HTML by default, perhaps something more
    // machinereadable?

    // TODO check response
    env::set_current_dir(&Path::new("data"));

    rocket::ignite()
        .mount("/", routes![root, files, puts])
        .launch();
}
