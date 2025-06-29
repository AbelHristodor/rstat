use std::time::Duration;

use http::{
    HeaderMap,
    header::{CONTENT_TYPE, HOST, USER_AGENT},
};
use uuid::Uuid;
use chrono::Utc;

use rstat_core::{Service, Kind, HttpChecker};

pub fn fixtures() -> Result<Vec<Service>, anyhow::Error> {
    let url = "http://localhost:5000/health";
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Rstat/Healtcheck".parse().unwrap());
    headers.insert(HOST, "localhost".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    
    let h2 = headers.clone();
    let h3 = headers.clone();
    let h4 = headers.clone();

    let http1 = HttpChecker {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: headers.into_iter().map(|(k, v)| (k.unwrap().to_string(), v.to_str().unwrap().to_string())).collect(),
        body: Some("hello".to_string()),
        timeout: 10,
        max_retries: 10,
    };

    let http2 = HttpChecker {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: h2.into_iter().map(|(k, v)| (k.unwrap().to_string(), v.to_str().unwrap().to_string())).collect(),
        body: Some("hello".to_string()),
        timeout: 10,
        max_retries: 10,
    };
    
    let http3 = HttpChecker {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: h3.into_iter().map(|(k, v)| (k.unwrap().to_string(), v.to_str().unwrap().to_string())).collect(),
        body: Some("hello".to_string()),
        timeout: 10,
        max_retries: 10,
    };
    
    let http4 = HttpChecker {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: h4.into_iter().map(|(k, v)| (k.unwrap().to_string(), v.to_str().unwrap().to_string())).collect(),
        body: Some("hello".to_string()),
        timeout: 10,
        max_retries: 10,
    };
    
    let now = Utc::now();
    let services = vec![
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service A"),
            kind: Kind::HTTP(http1),
            interval: Duration::new(30, 0), // 30 seconds
            next_run: now,
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service B"),
            kind: Kind::HTTP(http2),
            interval: Duration::new(60, 0), // 1 minute
            next_run: now,
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service C"),
            kind: Kind::HTTP(http3),
            interval: Duration::new(120, 0), // 2 minutes
            next_run: now,
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service D"),
            kind: Kind::HTTP(http4),
            interval: Duration::new(10, 0), // 10 seconds
            next_run: now,
        },
    ];
    Ok(services)
} 