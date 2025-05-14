use std::{
    collections::VecDeque,
    env,
    fs::File,
    io::{self, BufRead, Write},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime},
};

use reqwest::blocking::Client;

#[derive(Debug)]
struct WebsiteStatus {
    url: String,
    action_status: Result<u16, String>,
    response_time: Duration,
    timestamp: SystemTime,
}

fn fetch_status(client: &Client, url: &str, retries: u32) -> WebsiteStatus {
    let start = Instant::now();
    let mut attempts = 0;

    while attempts <= retries {
        let res = client.get(url).send();
        let elapsed = start.elapsed();

        match res {
            Ok(resp) => {
                return WebsiteStatus {
                    url: url.to_string(),
                    action_status: Ok(resp.status().as_u16()),
                    response_time: elapsed,
                    timestamp: SystemTime::now(),
                };
            }
            Err(e) if attempts == retries => {
                return WebsiteStatus {
                    url: url.to_string(),
                    action_status: Err(e.to_string()),
                    response_time: elapsed,
                    timestamp: SystemTime::now(),
                };
            }
            _ => {
                attempts += 1;
                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    unreachable!()
}

fn write_json(results: &[WebsiteStatus]) {
    let mut file = File::create("status.json").unwrap();
    writeln!(file, "[").unwrap();

    for (i, result) in results.iter().enumerate() {
        let action_status_str = match &result.action_status {
            Ok(code) => format!("\"action_status\": {{ \"Ok\": {} }}", code),
            Err(e) => format!("\"action_status\": {{ \"Err\": \"{}\" }}", e),
        };

        let timestamp_str = match result.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => format!("{}", duration.as_secs()),
            Err(_) => "0".to_string(),
        };

        writeln!(file, "  {{").unwrap();
        writeln!(file, "    \"url\": \"{}\",", result.url).unwrap();
        writeln!(file, "    {},", action_status_str).unwrap();
        writeln!(file, "    \"response_time_ms\": {},", result.response_time.as_millis()).unwrap();
        writeln!(file, "    \"timestamp\": \"{}\"", timestamp_str).unwrap();
        writeln!(file, "  }}{}", if i == results.len() - 1 { "" } else { "," }).unwrap();
    }

    writeln!(file, "]").unwrap();
}


fn parse_args() -> (Vec<String>, usize, u64, u32) {
    let args: Vec<String> = env::args().collect();
    let mut urls = vec![];
    let mut workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut timeout = 5;
    let mut retries = 0;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--file" => {
                i += 1;
                if i < args.len() {
                    if let Ok(lines) = read_lines(&args[i]) {
                        for line in lines.flatten() {
                            if !line.trim().is_empty() && !line.trim().starts_with('#') {
                                urls.push(line.trim().to_string());
                            }
                        }
                    }
                }
            }
            "--workers" => {
                i += 1;
                if i < args.len() {
                    workers = args[i].parse().unwrap_or(workers);
                }
            }
            "--timeout" => {
                i += 1;
                if i < args.len() {
                    timeout = args[i].parse().unwrap_or(timeout);
                }
            }
            "--retries" => {
                i += 1;
                if i < args.len() {
                    retries = args[i].parse().unwrap_or(retries);
                }
            }
            _ => {
                urls.push(args[i].clone());
            }
        }
        i += 1;
    }

    if urls.is_empty() {
        eprintln!("Usage: website_checker [--file sites.txt] [URL ...] [--workers N] [--timeout S] [--retries N]");
        std::process::exit(2);
    }

    (urls, workers, timeout, retries)
}

fn read_lines(path: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let (urls, worker_count, timeout, retries) = parse_args();

    let job_queue = Arc::new(Mutex::new(VecDeque::from(urls)));
    let (tx, rx) = mpsc::channel();

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .unwrap(),
    );

    let mut handles = vec![];

    for _ in 0..worker_count {
        let job_queue = Arc::clone(&job_queue);
        let tx = tx.clone();
        let client = Arc::clone(&client);

        let handle = thread::spawn(move || {
            loop {
                let url_opt = {
                    let mut queue = job_queue.lock().unwrap();
                    queue.pop_front()
                };

                match url_opt {
                    Some(url) => {
                        let status = fetch_status(&client, &url, retries);
                        tx.send(status).unwrap();
                    }
                    None => break,
                }
            }
        });

        handles.push(handle);
    }

    drop(tx); // Close channel

    let mut results = vec![];
    for status in rx {
        match &status.action_status {
            Ok(code) => println!("[{}] {} => {}", status.timestamp.elapsed().unwrap().as_secs(), status.url, code),
            Err(e) => {
                let msg = e.split(':').next().unwrap_or("Unknown error").trim();
                println!("[{}] {} => ERROR: {}", status.timestamp.elapsed().unwrap().as_secs(), status.url, msg);
            }   

        }
        results.push(status);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    write_json(&results);
}
