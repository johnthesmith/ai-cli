cargo build --release && \
mkdir -p ~/.local/bin && \
upx --best --ultra-brute target/release/ai && \
cp target/release/ai ~/.local/bin/ai
