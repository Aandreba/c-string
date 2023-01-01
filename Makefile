doc:
	cargo rustdoc --open --all-features -- --cfg docsrs

test:
	cargo test --test main --all-features -- --nocapture