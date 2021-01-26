use clap::Clap;
use std::net::TcpListener;
use std::time::{Instant, Duration};

fn available_port() -> std::io::Result<u16> {
    TcpListener::bind("localhost:0").map(|x| x.local_addr().unwrap().port())
}

#[derive(Clap)]
struct Opts {
    cluster_size: usize,
    command_size: u64,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let n = 16;
    let mut ports = vec![];
    for _ in 0..n {
        let port = available_port().unwrap();
        ports.push(port);
    }
    dbg!(&ports);
    for i in 0..n {
        tokio::spawn(async {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }
    // Set up the cluster

    // IO iteration
    let mut n = 0;
    let start_time = Instant::now();
    loop {
        // do IO
        tokio::time::sleep(Duration::from_millis(100)).await; // tmp
        n += 1;
        if Instant::now() - start_time >= Duration::from_secs(1) {
            break;
        }
    }
    let elapsed = Instant::now() - start_time;
    println!("response time per iteration: {}[ms]", (elapsed / n).as_millis());
}
