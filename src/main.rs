use clap::Clap;
use std::net::TcpListener;
use std::time::{Instant, Duration};
use lol_core::proto_compiled::{AddServerReq, raft_client::RaftClient};
use tonic::transport::channel::Endpoint;
use lol_core::compat::{RaftAppCompat, ToRaftApp};
use lol_core::{Config, Index, RaftCore, TunableConfig};

fn available_port() -> std::io::Result<u16> {
    TcpListener::bind("localhost:0").map(|x| x.local_addr().unwrap().port())
}

#[derive(Clap)]
struct Opts {
    #[clap(long, default_value = "10")]
    runtime: u64,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let n = 4;
    let mut ports = vec![];
    for _ in 0..n {
        let port = available_port().unwrap();
        ports.push(port);
    }

    // Launch the servers
    for i in 0..n {
        let port = ports[i];
        tokio::spawn(async move {
            let app = NoopApp {};
            let app = ToRaftApp::new(app);
            let storage = lol_core::storage::memory::Storage::new();
            let config = Config::new(format!("http://127.0.0.1:{}", port));
            let mut tunable = TunableConfig::default();
            tunable.compaction_interval_sec = 5;
            tunable.compaction_delay_sec = 1;
            let core = RaftCore::new(app, storage, config, tunable).await;
            let service = lol_core::make_service(core);
            let sock = format!("127.0.0.1:{}", port);
            let sock = sock.parse().unwrap();
            let mut builder = tonic::transport::Server::builder();
            builder.add_service(service).serve(sock).await.expect("Failed to start server");
        });
    }
    tokio::time::sleep(Duration::from_secs(1)).await;

    let leader_id = format!("http://127.0.0.1:{}", ports[0]);
    let endpoint = Endpoint::from_shared(leader_id.clone()).unwrap();
    let mut conn = RaftClient::connect(endpoint).await.unwrap();

    // Set up the cluster
    for i in 0..n {
        println!("add server[{}]", i);
        let id = format!("http://127.0.0.1:{}", ports[i]);
        let req = AddServerReq {
            id,
        };
        conn.add_server(req).await.expect("Failed to add server");
        println!("server[{}] added", i);
        tokio::time::sleep(Duration::from_secs(1)).await; // Safe guard
    }

    // IO iteration
    let buf = vec![0; 1048576];
    let mut n_resp = 0;
    let start_time = Instant::now();

    let mut term = 1;
    let mut term_n_resp = 0;
    let mut term_start_time = Instant::now();
    for i in 1.. {
        // do IO
        let res = conn.request_commit(lol_core::proto_compiled::CommitReq {
            core: false,
            message: buf.clone(),
        })
        .await;
        if res.is_ok() {
            term_n_resp += 1;
            n_resp += 1;
        } else {
            eprintln!("I/O failed ({})", i);
        }
        let now = Instant::now();
        if now - start_time >= Duration::from_secs(opts.runtime) {
            break;
        }
        let term_elapsed = now - term_start_time;
        if term_elapsed >= Duration::from_secs(1) && term_n_resp > 0 {
            eprintln!("[term {}] ave. response time: {}[ms]",  term, (term_elapsed / term_n_resp).as_millis());

            // Reset the counter
            term += 1;
            term_n_resp = 0;
            term_start_time = Instant::now();
        }
    }
    let elapsed = Instant::now() - start_time;
    eprintln!("[total] ave. response time: {}[ms]", (elapsed / n_resp).as_millis());
}

struct NoopApp {}
#[tonic::async_trait]
impl RaftAppCompat for NoopApp {
    async fn process_message(&self, _: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn apply_message(
        &self,
        _: &[u8],
        _: Index,
    ) -> anyhow::Result<(Vec<u8>, Option<Vec<u8>>)> {
        Ok((Vec::new(), None))
    }
    async fn install_snapshot(&self, _: Option<&[u8]>, _: Index) -> anyhow::Result<()> {
        Ok(())
    }
    async fn fold_snapshot(
        &self,
        _: Option<&[u8]>,
        _: Vec<&[u8]>,
    ) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }
}