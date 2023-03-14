build:
	docker build -t quantum-metachain .

stop:
	docker-compose -p quantum-metachain down --remove-orphans || true
.PHONY: stop

start:
	make stop
	cp docker/local/genesis_psk docker/local/alice/psk
	cp docker/local/genesis_psk docker/local/bob/psk
	docker-compose up
.PHONY: start
