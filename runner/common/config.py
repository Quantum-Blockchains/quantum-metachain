import json
from os import path, mkdir

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

default_config = {
    "local_peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
    "local_server_port": 5003,
    "external_server_port": 5004,
    "node_http_rpc_port": 9933,
    "psk_file_path": "psk1",
    "psk_sig_file_path": "psk1_sig",
    "node_key_file_path": ".node_key",
    "node_logs_path": "node.log",
    "recovery_check_interval": 50,
    "key_rotation_time": 5,
    "qrng_api_key": "api_key",
    "peers": {
        "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc": {
            "qkd": {
                "provider": "etsi014",
                "url": "http://localhost:9182/api/v1/keys/Alice1SAE",
                "client_cert_path": "certificates/12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc/qbck-client.crt",
                "cert_key_path": "certificates/12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc/qbck-client.key"
            },
            "server_addr": "http://localhost:5002"
        }
    }
}


class Config:
    def __init__(self, config_dict):
        for key, value in config_dict.items():
            if key.endswith("path") and value is not None:
                value = to_absolute(value)
            if key == "peers" and value is not None:
                value = self.process_peers(value)
            setattr(self, key, value)

    @staticmethod
    def from_json(json_data):
        return Config(json.loads(json_data, object_hook=lambda obj: obj))

    @staticmethod
    def process_peers(peers):
        for peer_id, peer_config in peers.items():
            qkd_config = peer_config['qkd']
            if 'client_cert_path' in qkd_config and qkd_config['client_cert_path'] is not None:
                qkd_config['client_cert_path'] = to_absolute(qkd_config['client_cert_path'])
            if 'cert_key_path' in qkd_config and qkd_config['cert_key_path'] is not None:
                qkd_config['cert_key_path'] = to_absolute(qkd_config['cert_key_path'])
        return peers


def to_absolute(*paths) -> str:
    return path.join(ROOT_DIR, *paths)


def init_config(config_path=None):
    if config_path is None:
        config = Config(default_config)
    else:
        with open(to_absolute(config_path), "r") as f:
            config = Config.from_json(f.read())

    global config_service
    config_service = ConfigService(config)


def create_node_info_dir():
    node_info_dir = to_absolute("node_info")
    if not path.exists(node_info_dir):
        mkdir(node_info_dir)

    peer_id_dir = path.join(node_info_dir, f'{config_service.config.local_peer_id}')
    if not path.exists(peer_id_dir):
        mkdir(peer_id_dir)

    logs_dir = path.join(peer_id_dir, 'logs')
    if not path.exists(logs_dir):
        mkdir(logs_dir)

    config_service.config.runner_logs_path = path.join(logs_dir, "runner.log")
    config_service.config.node_logs_path = path.join(logs_dir, "node.log")
    config_service.config.psk_sig_file_path = path.join(peer_id_dir, "psk_sig")


class ConfigService:
    def __init__(self, config):
        self.config = config


config_service = ConfigService(None)
