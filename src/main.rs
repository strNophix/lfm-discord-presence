use chrono::Duration;
use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let discord_app_id =
        env::var("DISCORD_APP_ID").expect("could not find `DISCORD_APP_ID` in env");
    let lfm_username =
        env::var("LASTFM_USERNAME").expect("could not find `LASTFM_USERNAME` in env");
    let lfm_api_key = env::var("LASTFM_API_KEY").expect("could not find `LASTFM_API_KEY` in env");

    let _ipc_client = Arc::new(Mutex::new(
        DiscordIpcClient::new(discord_app_id.as_str())
            .expect("failed to create Discord IPC-client"),
    ));
    let _ipc_client2 = Arc::clone(&_ipc_client);

    let mut lfm = lastfm_rs::Client::new(lfm_api_key.as_str());
    let user_url = format!("https://www.last.fm/user/{}", lfm_username);

    {
        let mut ipc_client = _ipc_client.lock().unwrap();
        ipc_client.connect().unwrap();
    }

    tokio::spawn(async move {
        loop {
            let tracks = lfm
                .recent_tracks(&lfm_username)
                .await
                .with_limit(1)
                .send()
                .await
                .expect("no recent track found");

            let last_track = &tracks.tracks[0];
            match &last_track.attrs {
                Some(_) => {
                    let details = format!("{} - {}", last_track.artist.name, last_track.name);
                    println!("Currently playing: {:#?}", details);
                    let state = format!("on {}", last_track.album.name);
                    let activity = Activity::new()
                        .assets(Assets::new().large_image(last_track.images[2].image_url.as_str()))
                        .details(details.as_str())
                        .state(state.as_str())
                        .buttons(vec![Button::new("Profile", &user_url)]);

                    {
                        let mut ipc_client = _ipc_client2.lock().unwrap();
                        ipc_client.set_activity(activity).unwrap();
                    }
                }
                None => {
                    println!("Current not playing...")
                }
            }
            thread::sleep(Duration::seconds(5).to_std().unwrap());
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            println!("Shutting down...");
            let mut ipc_client = _ipc_client.lock().unwrap();
            ipc_client.close().unwrap();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}
