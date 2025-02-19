use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct Person {
    id: u32,
    name: String,
}

type SharedPeople = Arc<Mutex<Vec<Person>>>;

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    people: SharedPeople,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/people") => {
            let people = people.lock().await;
            let json_body = to_string(&*people).unwrap();
            Ok(Response::new(full(json_body)))
        }
        (&Method::POST, "/people") => {
            use http_body_util::BodyExt;

            let body_bytes = req.collect().await?.to_bytes();

            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

            let new_person_data: serde_json::Value = match serde_json::from_str(&body_str) {
                Ok(data) => data,
                Err(_) => {
                    let mut response = Response::new(full("Invalid JSON format"));
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            let Some(name) = new_person_data["name"].as_str() else {
                let mut response = Response::new(full("Missing 'name' field"));
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            };

            if name.trim().is_empty() {
                let mut response = Response::new(full("'name' cannot be empty"));
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            }

            let mut people = people.lock().await;

            let new_id = people.last().map(|p| p.id + 1).unwrap_or(1);

            let new_person = Person {
                id: new_id,
                name: name.to_string(),
            };
            people.push(new_person);

            Ok(Response::new(full("Person added")))
        }
        (&Method::PUT, path) if path.starts_with("/people/") => {
            use http_body_util::BodyExt;

            let id_str = path.trim_start_matches("/people/");
            let id: u32 = match id_str.parse() {
                Ok(num) => num,
                Err(_) => {
                    let mut response = Response::new(full("Invalid ID format"));
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            let body_bytes = req.collect().await?.to_bytes();

            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

            let new_person_data: serde_json::Value = match serde_json::from_str(&body_str) {
                Ok(data) => data,
                Err(_) => {
                    let mut response = Response::new(full("Invalid JSON format"));
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            let Some(new_name) = new_person_data["name"].as_str() else {
                let mut response = Response::new(full("Missing 'name' field"));
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            };

            if new_name.trim().is_empty() {
                let mut response = Response::new(full("'name' cannot be empty"));
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            }

            let mut people = people.lock().await;

            if let Some(person) = people.iter_mut().find(|p| p.id == id) {
                person.name = new_name.to_string();
                Ok(Response::new(full("Person updated")))
            } else {
                let mut response = Response::new(full("Person not found"));
                *response.status_mut() = StatusCode::NOT_FOUND;
                Ok(response)
            }
        }

        (&Method::DELETE, path) if path.starts_with("/people/") => {
            let id_str = path.trim_start_matches("/people/");
            let id: u32 = match id_str.parse() {
                Ok(num) => num,
                Err(_) => {
                    let mut response = Response::new(full("Invalid ID format"));
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            let mut people = people.lock().await;

            let original_len = people.len();
            people.retain(|p| p.id != id);

            if people.len() < original_len {
                Ok(Response::new(full("Person deleted")))
            } else {
                let mut response = Response::new(full("Person not found"));
                *response.status_mut() = StatusCode::NOT_FOUND;
                Ok(response)
            }
        }

        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let people = Arc::new(Mutex::new(vec![
        Person {
            id: 1,
            name: "Alice".to_string(),
        },
        Person {
            id: 2,
            name: "Bob".to_string(),
        },
        Person {
            id: 3,
            name: "Charlie".to_string(),
        },
    ]));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let people = Arc::clone(&people);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, Arc::clone(&people))),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
