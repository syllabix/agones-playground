mod handler;

use std::io::{Error, ErrorKind};
use std::time::Duration;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use agones::Sdk;
use rand::Rng;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .map(|ps| ps.parse::<u16>().expect("Invalid PORT specified"))
        .unwrap_or(8080);

    let server = HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .route("/healthz", web::get().to(handler::health_check))
    })
    .bind((host, port))?
    .run();

    let game = start_game();
    let processes = tokio::join!(game, server);

    processes.0.map_err(|e| {
        log::error!("failed to start up the game process {}", e);
        Error::new(ErrorKind::Other, "game error")
    })?;

    processes.1.map_err(|e| {
        log::error!("failed to start up the web server {}", e);
        Error::new(ErrorKind::Other, "server error")
    })?;

    Ok(())
}

async fn start_game() -> Result<(), String> {
    let mut sdk = agones::Sdk::new(None, None)
        .await
        .map_err(|e| format!("unable to create a client sdk: {}", e))?;

    let _h = run_health_check(&sdk);

    sdk.set_label("orimboard", "gameserver-1")
        .await
        .map_err(|e| format!("failed to set_label(): {}", e))?;

    log::info!("marking server as ready");
    sdk.ready()
        .await
        .map_err(|e| format!("unable to mark server as ready: {}", e))?;

    log::info!("Setting as Reserved for 5 seconds");
    sdk.reserve(Duration::from_secs(5))
        .await
        .map_err(|e| format!("Could not run Reserve(): {}. Exiting!", e))?;
    log::info!("...Reserved");

    tokio::time::sleep(Duration::from_secs(6)).await;

    sdk.set_label("gs-session-ready", "true")
        .await
        .map_err(|e| format!("Could not label GameServer() as ready: {}. Exiting!", e))?;

    let id: u16 = rand::thread_rng().gen();

    sdk.set_label(format!("gs-{}", id), "space-id")
        .await
        .map_err(|e| format!("Could not label GameServer(): {}. Exiting!", e))?;

    let id: u16 = rand::thread_rng().gen();

    sdk.set_label(format!("gs-{}", id), "space-id")
        .await
        .map_err(|e| format!("Could not label GameServer(): {}. Exiting!", e))?;

    log::info!("Getting GameServer details...");
    let gameserver = sdk
        .get_gameserver()
        .await
        .map_err(|e| format!("Could not run GameServer(): {}. Exiting!", e))?;

    let status = gameserver.status.unwrap();
    log::info!("GameServer Status: {:?}", status);

    for i in 0..1000 {
        let time = i * 10;
        println!("Running for {} seconds", time);

        tokio::time::sleep(Duration::from_secs(10)).await;

        if i == 999 {
            println!("Shutting down...");
            sdk.shutdown()
                .await
                .map_err(|e| format!("Could not run Shutdown: {}. Exiting!", e))?;
            println!("...marked for Shutdown");
        }
    }

    Ok(())
}

fn run_health_check(sdk: &Sdk) -> tokio::sync::oneshot::Sender<()> {
    let health_tx = sdk.health_check();
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match health_tx.send(()).await {
                        Ok(_) => log::info!("health check ping ok"),
                        Err(e) => {
                            log::error!("game server health check failed: {}", e);
                            break;
                        },
                    }
                }

                _ = &mut rx => {
                    log::info!("health check task cancelled");
                    break;
                }

            }
        }
    });

    return tx;
}
