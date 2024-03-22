use reqwest::Client;
use std::time::{Instant, Duration};
use futures::future::join_all;

async fn is_server_available(url: &str) -> bool {
    let ping_url = format!("{}/ping", url);
    match reqwest::get(&ping_url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_url = "http://127.0.0.1:8080";
    if !is_server_available(server_url).await {
        eprintln!("Error: Server {} is not available", server_url);
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number_of_requests>\n pass the required number of requests when launching the application, e.g.: cargo run 13", args[0]);
        std::process::exit(1);
    }

    let n_requests: usize = args[1].parse()?;
    if n_requests < 1 || n_requests > 100 {
        eprintln!("Number of requests must be between 1 and 100");
        std::process::exit(1);
    }

    let mut handles = Vec::with_capacity(n_requests);
    let start_time = Instant::now();

    for _ in 0..n_requests {
        let handle = tokio::spawn(async move {
            let client = Client::new();
            let response = client.get(server_url).send().await?;
            Ok::<_, reqwest::Error>(response)
        });
        handles.push(handle);
    }

    let responses = join_all(handles).await;

    let mut min_response_time = Duration::from_secs(u64::MAX);
    let mut max_response_time = Duration::from_secs(0);
    let mut total_response_time = Duration::from_secs(0);
    let mut n_responses = 0;

    for response in responses {
        match response {
            Ok(resp) => {
                // Измеряем время между отправкой запроса и получением ответа
                let response_time = start_time.elapsed();
                if response_time < min_response_time {
                    min_response_time = response_time;
                }
                if response_time > max_response_time {
                    max_response_time = response_time;
                }
                total_response_time += response_time;
                n_responses += 1;
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }

    let avg_response_time = total_response_time / n_responses as u32;

    println!("Total requests: {}", n_requests);
    println!("Total responses: {}", n_responses);
    println!("Minimum response time: {:?}", min_response_time);
    println!("Maximum response time: {:?}", max_response_time);
    println!("Average response time: {:?}", avg_response_time);

    Ok(())
}
