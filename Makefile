serve:
	trunk serve

build:
	trunk build --release --dist docs --public-url 'soroban-fiddle'

check:
	cargo check
