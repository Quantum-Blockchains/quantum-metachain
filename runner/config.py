import json
import sys
from os import path

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
    "local_server_port": 5003,
    "external_server_port": 5004,
    "psk_file_path": "/tmp/psk1",
    "psk_sig_file_path": "/tmp/psk1_sig",
    "node_key_file_path": ".node_key",
    "key_rotation_time": 50,
    "qrng_api_key": "api_key",
    "peers": {
        "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc": {
            "qkd_addr": "http://212.244.177.99:9182/api/v1/keys/Alice1SAE",
            "server_addr": "http://localhost:5002"
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
