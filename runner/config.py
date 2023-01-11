import json
import sys
from os import path

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_server_port": 5001,
    "external_server_port": 5002,
    "psk_file_path": "/tmp/psk1",
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

    def abs_psk_file_path(self):
        return f"{ROOT_DIR}/{self.config['psk_file_path']}"


def abs_node_key_file_path():
    return f"{ROOT_DIR}/{config.config['node_key_file_path']}"


def abs_psk_sig_file_path():
    return f"{ROOT_DIR}/{config.config['psk_sig_file_path']}"


if len(sys.argv) < 2:
    config = Config()
elif sys.argv[1] != '--config':
    config = Config()
else:
    config_path = sys.argv[2]
    config = Config(config_path)



