use std::time::Duration;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use uuid::Uuid;
use rand::Rng;

use rstat_core::{Service, HealthCheckResult, Kind, HttpChecker, TcpChecker};

pub fn generate_services() -> Vec<Service> {
    vec![
        Service {
            id: Uuid::new_v4(),
            name: "API Gateway".to_string(),
            kind: Kind::HTTP(HttpChecker {
                url: "http://localhost:5000/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(30),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "Database Cluster".to_string(),
            kind: Kind::TCP(TcpChecker {
                host: "db.example.com".to_string(),
                port: 5432,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(60),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "Authentication Service".to_string(),
            kind: Kind::HTTP(HttpChecker {
                url: "http://localhost:5000/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(45),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "File Storage Service".to_string(),
            kind: Kind::HTTP(HttpChecker {
                url: "http://localhost:5000/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                timeout: 10,
                max_retries: 2,
            }),
            interval: Duration::from_secs(90),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "Email Service".to_string(),
            kind: Kind::HTTP(HttpChecker {
                url: "http://localhost:5000/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(120),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "CDN Edge Server".to_string(),
            kind: Kind::TCP(TcpChecker {
                host: "cdn.example.com".to_string(),
                port: 443,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(30),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "Redis Cache".to_string(),
            kind: Kind::TCP(TcpChecker {
                host: "redis.example.com".to_string(),
                port: 6379,
                timeout: 5,
                max_retries: 3,
            }),
            interval: Duration::from_secs(30),
            next_run: Utc::now(),
        },
        Service {
            id: Uuid::new_v4(),
            name: "Load Balancer".to_string(),
            kind: Kind::HTTP(HttpChecker {
                url: "http://localhost:5000/health".to_string(),
                method: "GET".to_string(),
                headers: std::collections::HashMap::new(),
                body: None,
                timeout: 3,
                max_retries: 1,
            }),
            interval: Duration::from_secs(15),
            next_run: Utc::now(),
        },
    ]
}

pub fn generate_health_check_results(
    service_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Vec<HealthCheckResult> {
    let mut rng = rand::thread_rng();
    let mut results = Vec::new();
    
    let mut current_time = start_date;
    
    while current_time <= end_date {
        // Limit to maximum 300 checks per day
        let checks_per_day = 300;
        
        for _ in 0..checks_per_day {
            // Add some randomness to check times (±5 minutes)
            let time_variation = rng.gen_range(-300..300);
            let check_time = current_time + ChronoDuration::seconds(time_variation);
            
            if check_time > end_date {
                break;
            }
            
            // Generate realistic success rate (95-99.9% for most services)
            let success_rate = match service_id.to_string().as_str() {
                s if s.contains("File Storage") => rng.gen_range(95.0..98.5), // More prone to issues
                s if s.contains("Email") => rng.gen_range(97.0..99.5), // Occasionally slow
                _ => rng.gen_range(99.0..99.9), // Most services are very reliable
            };
            
            let success = rng.gen_bool(success_rate / 100.0);
            
            // Generate realistic response times
            let base_latency = match service_id.to_string().as_str() {
                s if s.contains("Database") => rng.gen_range(5..25),
                s if s.contains("Redis") => rng.gen_range(1..10),
                s if s.contains("CDN") => rng.gen_range(5..20),
                s if s.contains("Load Balancer") => rng.gen_range(2..15),
                s if s.contains("File Storage") => rng.gen_range(50..200),
                s if s.contains("Email") => rng.gen_range(30..150),
                _ => rng.gen_range(10..50),
            };
            
            // Add some variation and occasional spikes
            let latency_variation = if rng.gen_bool(0.05) { // 5% chance of spike
                rng.gen_range(200..1000)
            } else {
                rng.gen_range(-10..30)
            };
            
            let response_time = (base_latency + latency_variation).max(1) * 1000; // Convert to microseconds
            
            // Generate HTTP status codes
            let code = if success {
                if rng.gen_bool(0.95) { 200 } else { 201 }
            } else {
                match rng.gen_range(0..4) {
                    0 => 500, // Internal server error
                    1 => 503, // Service unavailable
                    2 => 502, // Bad gateway
                    _ => 504, // Gateway timeout
                }
            };
            
            // Generate realistic error messages
            let message = if success {
                "OK".to_string()
            } else {
                match code {
                    500 => "Internal Server Error".to_string(),
                    503 => "Service Temporarily Unavailable".to_string(),
                    502 => "Bad Gateway".to_string(),
                    504 => "Gateway Timeout".to_string(),
                    _ => "Unknown Error".to_string(),
                }
            };
            
            results.push(HealthCheckResult {
                id: Uuid::new_v4(),
                success,
                code: code as u64,
                response_time: response_time as u128,
                message,
                created_at: check_time,
            });
        }
        
        current_time = current_time + ChronoDuration::days(1);
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_services() {
        let services = generate_services();
        assert_eq!(services.len(), 8);
        
        // Check that each service has a unique ID
        let ids: Vec<Uuid> = services.iter().map(|s| s.id).collect();
        let unique_ids: std::collections::HashSet<Uuid> = ids.into_iter().collect();
        assert_eq!(unique_ids.len(), 8);
    }
    
    #[test]
    fn test_generate_health_check_results() {
        let service_id = Uuid::new_v4();
        let start_date = Utc::now() - ChronoDuration::days(1);
        let end_date = Utc::now();
        
        let results = generate_health_check_results(service_id, start_date, end_date);
        
        // Should generate exactly 300 results for 1 day
        assert_eq!(results.len(), 300);
        
        // All results should be within the date range
        for result in &results {
            assert_eq!(result.created_at >= start_date, true);
            assert_eq!(result.created_at <= end_date, true);
        }
    }
} 