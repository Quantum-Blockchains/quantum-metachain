VENV=venv

build:
	docker build -t quantum-metachain .

stop:
	docker-compose -p quantum-metachain down --remove-orphans || true
.PHONY: stop

start:
	make stop
	cp docker/local/genesis_psk docker/local/alice/psk
	cp docker/local/genesis_psk docker/local/bob/psk
	cp docker/local/genesis_psk docker/local/dave/psk
	cp docker/local/genesis_psk docker/local/charlie/psk
	docker-compose up
.PHONY: start

venv:
	python3 -m venv ${VENV}
	. ./${VENV}/bin/activate
	venv/bin/pip3 install -r requirements.txt

config:
	${VENV}/bin/python3 runner/generate_config_node_wizard.py
