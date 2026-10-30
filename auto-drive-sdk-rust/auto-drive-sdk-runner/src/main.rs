use futures_util::StreamExt;
use reqwest::multipart;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

// TODO: don't do the color like this, this is just a remnant of the past .sh way of doing it

// ANSI color codes
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const GRAY: &str = "\x1b[90m";

const BASE_URL: &str = "https://cloud-autodrive.hugostechlab.com/autoscript";
// const BASE_URL: &str = "http://localhost:3030/autoscript";

#[tokio::main]
async fn main() {
    println!("{GREEN}[INFO]{RESET} Endpoint: {BASE_URL}");

    // --- Authentication Requirement ---
    let auth_token = load_auth_token().unwrap_or_else(|| {
        eprintln!("{RED}[ERROR]{RESET} Authentication token could not be loaded.\n");
        eprintln!("Please provide a valid .env file to authenticate.");
        eprintln!("----------------------------------------------------");
        let current_dir = env::current_dir().unwrap_or_default().display().to_string();
        let exe_dir = env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(Path::new(""))
            .display()
            .to_string();
        eprintln!("1. Create a file named exactly '.env' in one of these locations:");
        eprintln!("   - Current directory : {current_dir}");
        eprintln!("   - Executable dir    : {exe_dir}\n");
        eprintln!("2. Add your token to the file like this:");
        eprintln!("   AUTH_TOKEN=your_secure_token_here");
        eprintln!("----------------------------------------------------");
        eprintln!("Exiting.");
        process::exit(1);
    });

    println!("{GREEN}[INFO]{RESET} Authentication token loaded successfully.");

    // --- Parse Arguments & Find WASM ---
    let mut args: Vec<String> = env::args().collect();
    // args[0] is the runner executable itself.
    args.remove(0);

    let mut wasm_path: Option<PathBuf> = None;
    if !args.is_empty() {
        let potential_path = PathBuf::from(args[0].replace('\\', "/"));
        if potential_path.is_file() {
            wasm_path = Some(potential_path);
            args.remove(0);
        }
    }

    if wasm_path.is_none() {
        wasm_path = find_latest_wasm();
    }

    let wasm_file = match wasm_path {
        Some(path) if path.exists() => path,
        _ => {
            eprintln!("{RED}[ERROR]{RESET} WASM file not found");
            process::exit(1);
        }
    };

    let args_string = args.join(" ");

    println!("{GREEN}[INFO]{RESET} Running: {}", wasm_file.display());
    if !args_string.is_empty() {
        println!("{GREEN}[INFO]{RESET} Arguments passed: {}", args_string);
    }
    println!("{GREEN}[INFO]{RESET} Sending run wasm request to the cloud...");

    // Shared state for the run_uuid (so the cancellation thread can access it)
    let run_uuid: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // --- Setup Cancellation Handler (Ctrl+C) ---
    setup_ctrl_c_handler(run_uuid.clone(), auth_token.clone());

    // --- Read File and Prepare Request ---
    let wasm_bytes = fs::read(&wasm_file).expect("Failed to read WASM file");
    
    let file_part = multipart::Part::bytes(wasm_bytes)
        .file_name(wasm_file.file_name().unwrap().to_string_lossy().to_string())
        .mime_str("application/wasm")
        .unwrap();

    let form = multipart::Form::new()
        .part("file", file_part)
        .text("args", args_string);

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{BASE_URL}/run"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .multipart(form)
        .send()
        .await;

    match res {
        Ok(response) => {
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("{RED}[SERVER ERROR]{RESET} HTTP {status}: {body}");
                process::exit(1);
            }

            // --- Process Server-Sent Events (SSE) Stream ---
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        
                        // Process complete lines
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].to_string();
                            buffer = buffer[pos + 1..].to_string();
                            let line = line.trim_end_matches('\r');
                            
                            process_sse_line(&line, &run_uuid).await;
                        }
                    }
                    Err(e) => eprintln!("{RED}[ERROR]{RESET} Stream error: {e}"),
                }
            }
        }
        Err(e) => {
            eprintln!("{RED}[ERROR]{RESET} Request failed: {e}");
            process::exit(1);
        }
    }
}

async fn process_sse_line(line: &str, run_uuid: &Arc<Mutex<Option<String>>>) {
    if line.starts_with("event:") || line.is_empty() {
        return; // Ignore event framing lines and empty lines
    }

    if line.starts_with("data:") {
        let data = line["data:".len()..].trim_start();
        
        // Check for run_uuid
        if data.contains("\"run_uuid\"") {
            if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                if let Some(uuid) = parsed.get("run_uuid").and_then(|v| v.as_str()) {
                    let mut lock = run_uuid.lock().await;
                    *lock = Some(uuid.to_string());
                    println!("{GREEN}[INFO]{RESET} Assigned run UUID: {}", uuid);
                    return;
                }
            }
        }

        // Apply colors to log prefixes
        let colored = if data.starts_with("[ERROR]") {
            data.replacen("[ERROR]", &format!("{RED}[ERROR]{RESET}"), 1)
        } else if data.starts_with("[WARN]") {
            data.replacen("[WARN]", &format!("{YELLOW}[WARN]{RESET}"), 1)
        } else if data.starts_with("[INFO]") {
            data.replacen("[INFO]", &format!("{GREEN}[INFO]{RESET}"), 1)
        } else if data.starts_with("[DEBUG]") {
            data.replacen("[DEBUG]", &format!("{GRAY}[DEBUG]{RESET}"), 1)
        } else {
            data.to_string()
        };

        println!("{}", colored);
    } else {
        // Catch non-SSE plain text responses (like 413 error messages)
        println!("{RED}[SERVER ERROR]{RESET} {}", line);
    }
}

fn load_auth_token() -> Option<String> {
    let mut check_paths = vec![PathBuf::from(".env")];
    
    // Add executable directory to check paths
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            check_paths.push(parent.join(".env"));
        }
    }

    for path in check_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if line.starts_with("AUTH_TOKEN=") {
                    let token = line["AUTH_TOKEN=".len()..].trim();
                    // Strip quotes and carriage returns
                    return Some(token.trim_matches(|c| c == '"' || c == '\'' || c == '\r').to_string());
                }
            }
        }
    }
    None
}

fn find_latest_wasm() -> Option<PathBuf> {
    let search_dirs = [
        "target/wasm32-unknown-unknown/release",
        "target/wasm32-unknown-unknown/debug",
    ];

    let mut latest_path = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    for dir in search_dirs {
        let path = Path::new(dir);
        if !path.exists() { continue; }
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > latest_time {
                                latest_time = modified;
                                latest_path = Some(p);
                            }
                        }
                    }
                }
            }
        }
    }
    
    latest_path
}

fn setup_ctrl_c_handler(run_uuid: Arc<Mutex<Option<String>>>, auth_token: String) {
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        println!("\n{YELLOW}[WARN]{RESET} Cancelling.");

        let client = reqwest::Client::new();
        let uuid = run_uuid.lock().await.clone();

        let req = client
            .post(format!("{BASE_URL}/cancel"))
            .header("Authorization", format!("Bearer {auth_token}"));

        let res = if let Some(id) = uuid {
            println!("{GREEN}[INFO]{RESET} Sending cancel for run: {}", id);
            req.json(&serde_json::json!({ "run_uuid": id })).send().await
        } else {
            println!("{GREEN}[INFO]{RESET} No run_uuid received yet, sending general cancel.");
            req.send().await
        };

        if let Err(e) = res {
            println!("{YELLOW}[WARN]{RESET} Warning: cancel request failed: {}", e);
        } else if let Ok(r) = res {
            if !r.status().is_success() {
                println!("{YELLOW}[WARN]{RESET} Warning: cancel request returned status: {}", r.status());
            }
        }

        println!();
        process::exit(130);
    });
}