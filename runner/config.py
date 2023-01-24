import json
import sys
from os import path, mkdir


PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_peer_id": "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdB",
    "local_server_port": 5001,
    "external_server_port": 5002,
    "psk_file_path": "/tmp/psk1",
    "node_key_file_path": ".node_key",
    "key_rotation_time": 50,
    "qrng_api_key": "api_key",
    "peers": {
        "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV": {
            "qkd_addr": "http://localhost:9182/api/v1/keys/Alice1SAE/",
            "server_addr": "http://localhost:5004"
        }
    }
}


class Config:

    def __init__(self, config_path=None):
        if config_path is None:
            self.config = default_config
        else:
            with open(f"{ROOT_DIR}/{config_path}", "r") as f:
                self.config = json.load(f)
            if sys.argv[0] != "runner/runner_services_for_tests.py" and sys.argv[1] != "test" and sys.argv[0] != "runner/node_simulator.py":
                directory = path.join(ROOT_DIR, "tmp")
                if not path.exists(directory):
                    mkdir(directory)

                directory_node = path.join(directory, f'{self.config["local_peer_id"]}')
                if not path.exists(directory_node):
                    mkdir(directory_node)

                directory_logs = path.join(directory_node, 'logs')
                if not path.exists(directory_logs):
                    mkdir(directory_logs)

                self.config["path_logs_runner"] = f"{ROOT_DIR}/tmp/{self.config['local_peer_id']}/logs/runner.log"
                self.config["path_logs_node"] = f"{ROOT_DIR}/tmp/{self.config['local_peer_id']}/logs/node.log"
                self.config["psk_sig_file_path"] = f"tmp/{self.config['local_peer_id']}/psk_sig"

    def abs_psk_file_path(self):
        return f"{ROOT_DIR}/{self.config['psk_file_path']}"

    def abs_node_key_file_path(self):
        return f"{ROOT_DIR}/{self.config['node_key_file_path']}"

    def abs_psk_sig_file_path(self):
        return f"{ROOT_DIR}/{self.config['psk_sig_file_path']}"


if len(sys.argv) < 2:
    config = Config()
elif sys.argv[1] != '--config':
    config = Config()
else:
    config_path = sys.argv[2]
    config = Config(config_path)
