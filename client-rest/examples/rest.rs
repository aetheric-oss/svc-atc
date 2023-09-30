//! Example communication with this service

use hyper::{Body, Client, Method, Request, Response};
use hyper::{Error, StatusCode};
use svc_atc_client_rest::types::*;

fn evaluate(resp: Result<Response<Body>, Error>, expected_code: StatusCode) -> (bool, String) {
    let mut ok = true;
    let result_str: String = match resp {
        Ok(r) => {
            let tmp = r.status() == expected_code;
            ok &= tmp;
            println!("{:?}", r.body());

            r.status().to_string()
        }
        Err(e) => {
            ok = false;
            e.to_string()
        }
    };

    (ok, result_str)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NOTE: Ensure the server is running, or this example will fail.");

    let rest_port = std::env::var("SERVER_PORT_REST").unwrap_or_else(|_| "8000".to_string());
    // let host = std::env::var("SERVER_HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let host = "web-server".to_string();
    let url = format!("http://{host}:{rest_port}");
    let mut ok = true;
    let client = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(10))
        .build_http();

    // POST /ack/flight
    {
        let data = AckRequest {
            fp_id: "abcdef12".to_string(),
            status: AckStatus::Confirm,
        };

        let data_str = serde_json::to_string(&data).unwrap();
        let uri = format!("{}/ack/flight", url);
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri.clone())
            .header("content-type", "application/json")
            .body(Body::from(data_str))
            .unwrap();

        let resp = client.request(req).await;
        let (success, result_str) = evaluate(resp, StatusCode::OK);
        ok &= success;

        println!("{}: {}", uri, result_str);
    }

    if ok {
        println!("\u{1F9c1} All endpoints responded!");
    } else {
        eprintln!("\u{2620} Errors");
    }

    Ok(())
}
