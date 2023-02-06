import json
from os import path, mkdir


PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
    "local_server_port": 5003,
    "external_server_port": 5004,
    "psk_file_path": "tmp/psk1",
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


class InvalidConfigurationFile(Exception):
    "Raised when all required information is not provided in the configuration file."
    def __init__(self, key):
        self.message = f"No information provided in configuration file: {key}"
        super().__init__(self.message)


class Config:

    def __init__(self, config_path=None):
        if config_path is None:
            config_json = default_config
        else:
            with open(f"{ROOT_DIR}/{config_path}", "r") as f:
                config_json = json.load(f)

        if 'local_peer_id' in config_json:
            self.local_peer_id = config_json['local_peer_id']
        else:
            raise InvalidConfigurationFile('local_peer_id')

        if 'local_server_port' in config_json:
            self.local_server_port = config_json['local_server_port']
        else:
            raise InvalidConfigurationFile('local_server_port')

        if 'external_server_port' in config_json:
            self.external_server_port = config_json['external_server_port']
        else:
            raise InvalidConfigurationFile('external_server_port')

        if 'psk_file_path' in config_json:
            self.psk_file_path = config_json['psk_file_path']
        else:
            raise InvalidConfigurationFile('psk_file_path')

        if 'psk_sig_file_path' in config_json:
            self.psk_sig_file_path = config_json['psk_sig_file_path']
        else:
            raise InvalidConfigurationFile('psk_sig_file_path')

        if 'node_key_file_path' in config_json:
            self.node_key_file_path = config_json['node_key_file_path']
        else:
            raise InvalidConfigurationFile('node_key_file_path')

        if 'key_rotation_time' in config_json:
            self.key_rotation_time = config_json['key_rotation_time']
        else:
            raise InvalidConfigurationFile('key_rotation_time')

        if 'qrng_api_key' in config_json:
            self.qrng_api_key = config_json['qrng_api_key']
        else:
            raise InvalidConfigurationFile('qrng_api_key')

        if 'peers' in config_json:
            self.peers = config_json['peers']
        else:
            raise InvalidConfigurationFile('peers')

        if 'path_logs_node' in config_json:
            self.path_logs_node = config_json['path_logs_node']

    def abs_psk_file_path(self):
        return f"{ROOT_DIR}/{self.psk_file_path}"

    def abs_node_key_file_path(self):
        return f"{ROOT_DIR}/{self.node_key_file_path}"

    def abs_psk_sig_file_path(self):
        return f"{ROOT_DIR}/{self.psk_sig_file_path}"

    def abs_log_node_file_path(self):
        return f"{ROOT_DIR}/{self.path_logs_node}"


def create_directory_for_logs_and_other_files_of_node():
    directory = path.join(ROOT_DIR, "info_of_nodes")
    if not path.exists(directory):
        mkdir(directory)

    directory_node = path.join(directory, f'{config_service.current_config.local_peer_id}')
    if not path.exists(directory_node):
        mkdir(directory_node)

    directory_logs = path.join(directory_node, 'logs')
    if not path.exists(directory_logs):
        mkdir(directory_logs)

    config_service.current_config.path_logs_runner = f"{ROOT_DIR}/info_of_nodes/{config_service.current_config.local_peer_id}/logs/runner.log"
    config_service.current_config.path_logs_node = f"{ROOT_DIR}/info_of_nodes/{config_service.current_config.local_peer_id}/logs/node.log"
    config_service.current_config.psk_sig_file_path = f"info_of_nodes/{config_service.current_config.local_peer_id}/psk_sig"


class ConfigService:
    def __init__(self, config):
        self.current_config = config


config_service = ConfigService(None)
