travis: doc
	mv target/doc docs
	rustdoc README.md -o docs
	mv docs/README.html docs/index.html

doc:
	cargo doc --no-deps
