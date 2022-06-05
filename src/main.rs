use actix_cors::Cors;
use actix_web::{get, post};
use actix_web::{http, web, App, HttpServer, ResponseError};
use dotenv::dotenv;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Deserialize, Serialize)]
struct FormData {
    channel: String,
    message: String,
}
#[derive(Debug)]
enum CustomError {
    SlackResponseError,
    BodyExtractionError,
    ConversionError,
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::SlackResponseError => {
                write!(f, "There was an error in handling the response from Slack")
            }
            CustomError::BodyExtractionError => write!(f, "Unable to extract response body"),
            CustomError::ConversionError => write!(f, "Unable to convert body to json"),
        }
    }
}

impl ResponseError for CustomError {}

///Helper for processing requests sent to slack
async fn process_response(
    response: reqwest::Result<reqwest::Response>,
) -> actix_web::Result<web::Json<Value>, CustomError> {
    //Check the response and return if it errors
    if let Err(e) = response {
        println!("{}", e);
        return Err(CustomError::SlackResponseError);
    }
    //Get the body of the response
    if let Ok(body) = response.unwrap().text().await {
        //Try to convert it to json
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            println!("Error is {}", e);
            CustomError::ConversionError
        })?;
        //If all went well send the json as the response
        Ok(web::Json(json))
    } else {
        Err(CustomError::BodyExtractionError)
    }
}

/*****************************************HANDLERS***************************/

///Route that sends a message to the slack api
#[post("/send-message")]
async fn send_message(
    form: web::Form<FormData>,
    client: web::Data<reqwest::Client>,
) -> actix_web::Result<web::Json<Value>, CustomError> {
    println!("form: {:?}", form.message);
    let message = &form.message;
    let channel = &form.channel;

    //Make a hashmap of the stuff we need to send in a form to slack
    let mut body = HashMap::new();
    body.insert("channel", channel);
    body.insert("text", message);

    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .form(&body)
        .send()
        .await;
    process_response(res).await
}

#[get("/users")]
async fn get_users(
    client: web::Data<reqwest::Client>,
) -> actix_web::Result<web::Json<Value>, CustomError> {
    //Contact slack api
    let res = client.get("https://slack.com/api/users.list").send().await;
    //Check the response and return if it errors
    if let Err(e) = res {
        println!("{}", e);
        return Err(CustomError::SlackResponseError);
    }
    process_response(res).await
}

#[get("/conversations/{channel_id}")]
async fn get_conversation_info(
    path: web::Path<String>,
    client: web::Data<reqwest::Client>,
) -> actix_web::Result<web::Json<Value>, CustomError> {
    let res = client
        .get(format!(
            "https://slack.com/api/conversations.info?channel={}",
            path.into_inner()
        ))
        .send()
        .await;
    process_response(res).await
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    //Set the headers for the client builder, will need to be overriden if they mismatch
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Content-type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    headers.insert(
        "Authorization",
        dotenv::var("BOT_TOKEN")
            .unwrap_or(String::new())
            .parse()
            .unwrap(),
    );

    //Build the client and wrap it in web::Data so we can access it from multiple threads
    let client_builder = reqwest::ClientBuilder::new().default_headers(headers);
    let client = web::Data::new(client_builder.build().unwrap());

    HttpServer::new(move || {
        App::new()
            .wrap(setup_cors())
            .app_data(client.clone())
            .service(get_conversation_info)
            .service(get_users)
            .service(send_message)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// Return cors configuration for the project
fn setup_cors() -> Cors {
    Cors::default()
        .send_wildcard()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}
