devnet:
	cd cli && cargo run

nuke:
	rm -rf .devnet 
	rm -rf optimism optimism-rs
	rm .stack
