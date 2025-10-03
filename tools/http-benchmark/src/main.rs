#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(not(target_os = "none"))]
fn main() {
    use curl::easy::Easy;
    use std::env;
    use std::io::Write;
    use std::time::{Duration, Instant};

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <ip-address>", args[0]);
        std::process::exit(1);
    }
    let ip = &args[1];
    let url = format!("http://{}", ip);
    let runtime = Duration::from_secs(60);
    let start_time = Instant::now();

    let mut times = Vec::new();
    let mut count = 0u128;
    let mut total_micros = 0u128;
    while start_time.elapsed() < runtime {
        let mut easy = Easy::new();
        easy.url(&url).unwrap();
        easy.write_function(|data| Ok(data.len())).unwrap();
        let req_start = Instant::now();
        match easy.perform() {
            Ok(_) => {
                let duration = req_start.elapsed();
                let micros = duration.as_micros();
                times.push(micros);
                count += 1;
                total_micros += micros;
                let avg = total_micros / count;
                print!(
                    "\rRequest {}: {} µs round trip - Average: {} µs",
                    count, micros, avg
                );
                std::io::stdout().flush().unwrap();
            }
            Err(e) => eprintln!("\nRequest {} failed: {}", count + 1, e),
        }
    }
    println!("\nTotal sequential requests in 1 minute: {}", count);

    // Percentile calculations
    if !times.is_empty() {
        times.sort_unstable();
        let len = times.len();

        let idx = |percent: f64| ((percent * len as f64).ceil() as usize).saturating_sub(1);

        let one_percent_idx = idx(0.01);
        let ten_percent_idx = idx(0.10);
        let one_percent_high_idx = len - idx(0.01) - 1;
        let ten_percent_high_idx = len - idx(0.10) - 1;

        println!("1% low: {} µs", times[one_percent_idx]);
        println!("10% low: {} µs", times[ten_percent_idx]);
        println!("1% high: {} µs", times[one_percent_high_idx]);
        println!("10% high: {} µs", times[ten_percent_high_idx]);
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
