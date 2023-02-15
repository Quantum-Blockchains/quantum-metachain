import json
from os import path, mkdir


PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
    "local_server_port": 5003,
    "external_server_port": 5004,
    "psk_file_path": "psk1",
    "psk_sig_file_path": "psk1_sig",
    "node_key_file_path": ".node_key",
    "node_logs_path": "node.log",
    "key_rotation_time": 1,
    "qrng_api_key": "api_key",
    "peers": {
        "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW": {
            "qkd_addr": "http://212.244.177.99:8182/api/v1/keys/Bob1SAE",
            "server_addr": "http://localhost:5004"
        }
    }
}


class InvalidConfigurationFile(Exception):
    "Raised when all required information is not provided in the configuration file."
    def __init__(self, key):
        self.message = f"No information provided in configuration file: {key}"
        super().__init__(self.message)


class Config:

    def __init__(self, local_peer_id, local_server_port, external_server_port, psk_file_path, psk_sig_file_path,
                 node_key_file_path, key_rotation_time, qrng_api_key, node_logs_path, peers):
        self.local_peer_id = local_peer_id
        self.local_server_port = local_server_port
        self.external_server_port = external_server_port
        self.psk_file_path = psk_file_path
        self.psk_sig_file_path = psk_sig_file_path
        self.node_key_file_path = node_key_file_path
        self.key_rotation_time = key_rotation_time
        self.qrng_api_key = qrng_api_key
        self.node_logs_path = node_logs_path
        self.peers = peers

    def abs_psk_file_path(self):
        return f"{ROOT_DIR}/{self.psk_file_path}"

    def abs_node_key_file_path(self):
        return f"{ROOT_DIR}/{self.node_key_file_path}"

    def abs_psk_sig_file_path(self):
        return f"{ROOT_DIR}/{self.psk_sig_file_path}"

    def abs_log_node_file_path(self):
        return f"{ROOT_DIR}/{self.node_logs_path}"


def custom_config_decoder(obj):
    if '__type__' in obj and obj['__type__'] == 'Config':
        try:
            return Config(
                obj["local_peer_id"],
                obj["local_server_port"],
                obj["external_server_port"],
                obj["psk_file_path"],
                obj["psk_sig_file_path"],
                obj['node_key_file_path'],
                obj['key_rotation_time'],
                obj['qrng_api_key'],
                obj['node_logs_path'],
                obj['peers']
            )
        except KeyError as e:
            raise InvalidConfigurationFile(e)
    return obj


def init_config(config_path=None):
    if config_path is None:
        config = Config(
            default_config["local_peer_id"],
            default_config["local_server_port"],
            default_config["external_server_port"],
            default_config["psk_file_path"],
            default_config["psk_sig_file_path"],
            default_config['node_key_file_path'],
            default_config['key_rotation_time'],
            default_config['qrng_api_key'],
            default_config['node_logs_path'],
            default_config['peers']
        )
    else:
        with open(f"{ROOT_DIR}/{config_path}", "r") as f:
            config = json.load(f, object_hook=custom_config_decoder)

    global config_service
    config_service = ConfigService(config)


def create_node_info_dir():
    directory = path.join(ROOT_DIR, "node_info")
    if not path.exists(directory):
        mkdir(directory)

    directory_node = path.join(directory, f'{config_service.current_config.local_peer_id}')
    if not path.exists(directory_node):
        mkdir(directory_node)

    directory_logs = path.join(directory_node, 'logs')
    if not path.exists(directory_logs):
        mkdir(directory_logs)

    config_service.current_config.runner_logs_path = f"{ROOT_DIR}/node_info/{config_service.current_config.local_peer_id}/logs/runner.log"
    config_service.current_config.node_logs_path = f"{ROOT_DIR}/node_info/{config_service.current_config.local_peer_id}/logs/node.log"
    config_service.current_config.psk_sig_file_path = f"node_info/{config_service.current_config.local_peer_id}/psk_sig"


class ConfigService:
    def __init__(self, config):
        self.current_config = config


config_service = ConfigService(None)
