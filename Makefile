build-binary:
	mkdir -p bin
	cd cli && cargo build --release && cp ./target/release/op-up-cli ../bin/op-up

devnet:
	./bin/op-up

nuke:
	rm -rf .devnet
	rm .stack
