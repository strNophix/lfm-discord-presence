use chrono::{Duration, Local, Utc};
use discord_rich_presence::{
    activity::{Activity, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};
use tokio::{
    signal,
    sync::oneshot::{self, channel},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let discord_app_id = env::var("DISCORD_APP_ID").expect("Missing DISCORD_APP_ID in env");
    let lfm_username = env::var("LASTFM_USERNAME").expect("Missing LASTFM_USERNAME in env");
    let lfm_api_key = env::var("LASTFM_API_KEY").expect("Missing LASTFM_API_KEY in env");

    let (send_stop, mut recv_stop) = channel::<()>();

    let mut lfm = lastfm_rs::Client::new(lfm_api_key.as_str());

    let _ipc_client = Arc::new(Mutex::new(
        DiscordIpcClient::new(discord_app_id.as_str())
            .expect("failed to create Discord IPC-client"),
    ));
    let _ipc_client2 = Arc::clone(&_ipc_client);

    let mut ipc_client = _ipc_client.lock().unwrap();
    ipc_client.connect().unwrap();
    drop(ipc_client);

    tokio::spawn(async move {
        let user_url = format!("https://www.last.fm/user/{}", lfm_username);
        let started_at = Utc::now();
        loop {
            match recv_stop.try_recv() {
                Err(oneshot::error::TryRecvError::Empty) => {
                    let tracks_result = lfm
                        .recent_tracks(&lfm_username)
                        .await
                        .with_limit(1)
                        .send()
                        .await;

                    if let Ok(tracks) = tracks_result {
                        let last_track = &tracks.tracks[0];
                        match &last_track.attrs {
                            Some(_) => {
                                let details =
                                    format!("{} - {}", last_track.artist.name, last_track.name);
                                println!(
                                    "{} Currently playing: {:#?}",
                                    Local::now().format("%Y/%m/%d %H:%M:%S"),
                                    details
                                );
                                let state = format!("on {}", last_track.album.name);
                                let activity = Activity::new()
                                    .assets(
                                        Assets::new()
                                            .large_image(last_track.images[2].image_url.as_str())
                                            .large_text(&details),
                                    )
                                    .details(details.as_str())
                                    .state(state.as_str())
                                    .timestamps(Timestamps::new().start(started_at.timestamp()))
                                    .buttons(vec![Button::new("Profile", &user_url)]);

                                let mut ipc_client = _ipc_client2.lock().unwrap();
                                ipc_client.set_activity(activity).unwrap();
                                drop(ipc_client);
                            }
                            None => {
                                println!(
                                    "{}: Current not playing...",
                                    Local::now().format("%Y/%m/%d %H:%M:%S"),
                                );
                            }
                        }
                    }
                    thread::sleep(Duration::seconds(5).to_std().unwrap());
                }
                _ => {
                    break;
                }
            }
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            println!("Shutting down...");
            send_stop.send(()).unwrap();
            let mut ipc_client = _ipc_client.lock().unwrap();
            ipc_client.close().unwrap();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}
