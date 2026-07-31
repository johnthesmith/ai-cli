cargo build --release 2>&1 && \
mkdir -p ~/.local/bin && \
cp target/release/ai ~/.local/bin/ai
