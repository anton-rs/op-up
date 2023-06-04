devnet:
	cd cli && cargo run

devnet-stop:
	cd docker && docker-compose down

nuke:
	rm -rf .devnet
	rm .stack
