load-dotenv:
    set -a; source .env; set +a

build-release:
    cross build --target x86_64-pc-windows-gnu --release
    cross build --target x86_64-unknown-linux-gnu --release