use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const URL: &str = "http://192.168.0.180/logs";

// Total number of requests to make.
const TOTAL_REQUESTS: usize = 100;

// Maximum number of requests in flight at once.
const CONCURRENCY: usize = 2;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .http1_only()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(1))
        .pool_max_idle_per_host(CONCURRENCY)
        .build()
        .unwrap();

    let semaphore = Arc::new(Semaphore::new(CONCURRENCY));

    let start = Instant::now();

    let mut handles = Vec::with_capacity(TOTAL_REQUESTS);

    for i in 0..TOTAL_REQUESTS {
        let client = client.clone();
        let semaphore = semaphore.clone();

        handles.push(tokio::spawn(async move {
            // Limit the number of requests in flight.
            let _permit = semaphore.acquire().await.unwrap();

            let request = client.get(URL).build().unwrap();
                        
            // Print the outgoing request details
            println!("=== OUTGOING REQUEST ===");
            println!("Method: {}", request.method());
            println!("URL: {}", request.url());
            println!("Headers: {:#?}", request.headers());
            println!("========================");

            // Now send it
            let result = client.execute(request).await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    if !status.is_success() {
                        return Err(format!(
                            "request {i}: HTTP {status}"
                        ));
                    }

                    // Verify the response actually matches the OpenAPI schema:
                    // array[string]
                    let body: serde_json::Value = response
                        .json()
                        .await
                        .map_err(|e| format!("request {i}: invalid JSON: {e}"))?;

                    let valid = body
                        .as_array()
                        .map(|array| {
                            array.iter().all(|item| item.is_string())
                        })
                        .unwrap_or(false);

                    if !valid {
                        return Err(format!(
                            "request {i}: response is not an array of strings"
                        ));
                    }

                    Ok(())
                }

                Err(e) => Err(format!(
                    "request {i}: {e:?}"
                )),
            }
        }));
    }

    let mut successful = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(()) => successful += 1,
            Err(error) => {
                failed += 1;

                // Don't print 100,000 errors if the server dies.
                if failed <= 20 {
                    eprintln!("{error}");
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let requests_per_second =
        TOTAL_REQUESTS as f64 / elapsed.as_secs_f64();

    println!();
    println!("========== Stress Test ==========");
    println!("Requests:     {TOTAL_REQUESTS}");
    println!("Concurrency:  {CONCURRENCY}");
    println!("Successful:   {successful}");
    println!("Failed:       {failed}");
    println!("Duration:     {:.2}s", elapsed.as_secs_f64());
    println!("Throughput:   {:.2} req/s", requests_per_second);
    println!("=================================");
}

