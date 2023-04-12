from os import path
import copy


from common.config import Config, init_config, config_service, ROOT_DIR

# Sample configuration for testing purposes
test_config = {
    "local_peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
    "local_server_port": 6000,
    "external_server_port": 6001,
    "psk_file_path": "psk1_test",
    "psk_sig_file_path": "psk1_sig_test",
    "node_key_file_path": ".node_key_test",
    "node_logs_path": "node.log_test",
    "key_rotation_time": 7,
    "qrng_api_key": "api_key_test",
    "peers": {
        "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc": {
            "qkd_addr": "http://localhost:9182/api/v1/keys/Alice1SAE_test",
            "qkd_cert_path": "certificates/qbck-client_test.crt",
            "qkd_cert_key_path": "certificates/qbck-client_test.key",
            "server_addr": "http://localhost:6002"
        }
    }
}


def test_config_initialization():
    config = Config(copy.deepcopy(test_config))
    assert config.local_peer_id == test_config["local_peer_id"]
    assert config.local_server_port == test_config["local_server_port"]
    assert config.external_server_port == test_config["external_server_port"]
    assert config.key_rotation_time == test_config["key_rotation_time"]
    assert config.qrng_api_key == test_config["qrng_api_key"]

    # Paths in config are absolute
    assert config.psk_file_path == path.join(ROOT_DIR, test_config["psk_file_path"])
    assert config.psk_sig_file_path == path.join(ROOT_DIR, test_config["psk_sig_file_path"])
    assert config.node_key_file_path == path.join(ROOT_DIR, test_config["node_key_file_path"])
    assert config.node_logs_path == path.join(ROOT_DIR, test_config["node_logs_path"])


def test_peers_config_initialization():
    config = Config(copy.deepcopy(test_config))
    test_peer = test_config["peers"]["12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"]
    config_peer = config.peers["12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"]

    assert config_peer["qkd_addr"] == test_peer["qkd_addr"]
    assert config_peer["server_addr"] == test_peer["server_addr"]

    # Paths in config are absolute
    assert config_peer["qkd_cert_path"] == path.join(ROOT_DIR, test_peer["qkd_cert_path"])
    assert config_peer["qkd_cert_key_path"] == path.join(ROOT_DIR, test_peer["qkd_cert_key_path"])


def test_init():
    init_config(None)
    assert config_service.config is not None
