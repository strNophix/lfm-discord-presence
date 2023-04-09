load-dotenv:
    set -a; source ./.env; set +a

build-release:
    cargo build --target x86_64-unknown-linux-gnu --release

install:
    just build-release
    sudo cp ./target/x86_64-unknown-linux-gnu/release/lfm-discord-presence /usr/local/bin

    mkdir -p ~/.config/systemd/user
    cp ./systemd/lfm-discord-presence.service ~/.config/systemd/user
    
    sed -i "s/LASTFM_USERNAME/$LASTFM_USERNAME/g" ~/.config/systemd/user/lfm-discord-presence.service

    systemctl enable --user lfm-discord-presence.service --now

uninstall:
    systemctl disable --user lfm-discord-presence.service --now
    sudo rm /usr/local/bin/lfm-discord-presence

reinstall:
    just uninstall
    just install
