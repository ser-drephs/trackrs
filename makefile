
.PHONY: debug build-release release-linux-musl test clippy clippy-pedantic install install-debug

ARGS=-l

debug:
	RUST_BACKTRACE=true cargo run -- ${ARGS}

build:
	cargo build

build-ci:
	cargo build --locked

build-release:
	cargo build --release --offline

build-release-linux-musl:
	cargo build --release --target x86_64-unknown-linux-gnu

build-release-win:
	cargo build --release --target x86_64-pc-windows-gnu

build-release-mac:
	cargo build --release --target x86_64-apple-darwin

release-linux:
	cargo build --offline --release --target x86_64-unknown-linux-gnu
	mkdir -p release/linux-gnu
	mv ./target/x86_64-unknown-linux-gnu/release/trackrs ./release/linux-gnu/trackrs 2>/dev/null; true
	# tar -C ./target/x86_64-unknown-linux-gnu/release/ -czvf ./release/trackrs-linux-gnu.tar.gz ./trackrs
	ls -l ./release

release-win:
	cargo build --offline --release --target x86_64-pc-windows-gnu
	mkdir -p release/win
	mv ./target/x86_64-pc-windows-gnu/release/trackrs.exe ./release/win/trackrs.exe 2>/dev/null; true
	ls -l ./release

release-mac:
	cargo build --offline --release --target x86_64-apple-darwin
	mkdir -p release/mac
	mv ./target/x86_64-apple-darwin/release/trackrs ./release/mac/trackrs 2>/dev/null;
	# tar -C ./target/x86_64-apple-darwin/release/ -czvf ./release/trackrs-mac.tar.gz ./trackrs
	ls -l ./release

test:
	cargo test --workspace

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --workspace --all-features

check:
	cargo check --bins --release --target x86_64-unknown-linux-gnu
	cargo check --bins --release --target x86_64-pc-windows-gnu
	# cargo check --locked --bins --release --target x86_64-apple-darwin

deny:
	cargo deny check

install:
	cargo install --path "." --offline

licenses:
	cargo bundle-licenses --format yaml --output THIRDPARTY.yaml
