import json
import sys
from os import path, mkdir


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
            "qkd_addr": "http://localhost:9182/api/v1/keys/Alice1SAE",
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

    def abs_log_node_file_path(self):
        return f"{ROOT_DIR}/{self.config['path_logs_node']}"


if len(sys.argv) == 1:
    config = Config()
elif sys.argv[1] != '--config':
    config = Config()
else:
    config_path = sys.argv[2]
    config = Config(config_path)


def create_directory_for_logs_and_other_files_of_node():
    directory = path.join(ROOT_DIR, "info_of_nodes")
    if not path.exists(directory):
        mkdir(directory)

    directory_node = path.join(directory, f'{config.config["local_peer_id"]}')
    if not path.exists(directory_node):
        mkdir(directory_node)

    directory_logs = path.join(directory_node, 'logs')
    if not path.exists(directory_logs):
        mkdir(directory_logs)

    config.config["path_logs_runner"] = f"{ROOT_DIR}/info_of_nodes/{config.config['local_peer_id']}/logs/runner.log"
    config.config["path_logs_node"] = f"{ROOT_DIR}/info_of_nodes/{config.config['local_peer_id']}/logs/node.log"
    config.config["psk_sig_file_path"] = f"info_of_nodes/{config.config['local_peer_id']}/psk_sig"
