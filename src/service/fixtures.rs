use std::time::Duration;

use http::{
    HeaderMap,
    header::{CONTENT_TYPE, HOST, USER_AGENT},
};
use uuid::Uuid;

use crate::healthcheck::{self, Kind};

use super::Service;

pub fn fixtures() -> Result<Vec<Service>, anyhow::Error> {
    let url = "http://localhost:5000/health";
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Rstat/Healtcheck".parse().unwrap());
    headers.insert(HOST, "localhost".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    
    let h2 = headers.clone();
    let h3 = headers.clone();
    let h4 = headers.clone();

    let http1 = healthcheck::http::new(url.into(), None)
        .unwrap()
        .headers(headers)
        .retries(10)
        .body("hello".into())
        .timeout(10);

    let http2 = healthcheck::http::new(url.into(), None)
        .unwrap()
        .headers(h2)
        .retries(10)
        .body("hello".into())
        .timeout(10);
    
    let http3 = healthcheck::http::new(url.into(), None)
        .unwrap()
        .headers(h3)
        .retries(10)
        .body("hello".into())
        .timeout(10);
    
    let http4 = healthcheck::http::new(url.into(), None)
        .unwrap()
        .headers(h4)
        .retries(10)
        .body("hello".into())
        .timeout(10);
    
    let services = vec![
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service A"),
            kind: Kind::HTTP(http1),
            interval: Duration::new(30, 0), // 30 seconds
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service B"),
            kind: Kind::HTTP(http2),
            interval: Duration::new(60, 0), // 1 minute
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service C"),
            kind: Kind::HTTP(http3),
            interval: Duration::new(120, 0), // 2 minutes
        },
        Service {
            id: Uuid::new_v4(),
            name: String::from("Service D"),
            kind: Kind::HTTP(http4),
            interval: Duration::new(10, 0), // 10 seconds
        },
    ];
    Ok(services)
}
