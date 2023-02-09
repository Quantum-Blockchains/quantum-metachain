from unittest.mock import patch

import pytest

import node
from node import Node, NodeService
from web.local_server import rotate_pre_shared_key
import common.config
import common.file

psk = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a"
signature = "fe2550d4baf581af5f9cc9428e425b093fd4777fdfeb7d00a9b52261754a56ec5034dd0fe3570d9d7f7a21b9d2d2007cba1afe773430dbd79b7c0cf37a55e803"
common.config.config_service = common.config.ConfigService(common.config.Config())
common.file.psk_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_file_path())
common.file.psk_sig_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_psk_sig_file_path())
common.file.node_key_file_manager = common.file.FileManager(common.config.config_service.current_config.abs_node_key_file_path())

@pytest.fixture()
def before_each():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()


@patch("core.pre_shared_key.generate_psk_from_qrng", return_value=psk)
@patch('common.file.node_key_file_manager.read',
       return_value="df432c8e967aa21fdd287d3ea61fa85640a8309577f65b4ea78d49d514661654")
@patch('common.file.psk_file_manager.create')
@patch('common.file.psk_sig_file_manager.create')
@patch('common.file.psk_sig_file_manager.remove')
def test_rotate_pre_shared_key_when_local_peer_is_chosen(psk_sig_remove, psk_sig_create, psk_create, node_key_read,
                                                         generate_psk_from_qrng, before_each):
    body = {
        "is_local_peer": True,
        "peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    }

    rotate_pre_shared_key(body)

    generate_psk_from_qrng.assert_called()
    node_key_read.assert_called()
    psk_create.assert_called_with(psk)
    psk_sig_create.assert_called_with(signature)
    psk_sig_remove.assert_called()


@patch("core.pre_shared_key.get_psk_from_peers", return_value=("c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a", "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"))
def test_rotate_pre_shared_key_when_other_peer_is_chosen(get_psk_from_peers, before_each):
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    body = {
        "is_local_peer": False,
        "peer_id": peer_id
    }
    rotate_pre_shared_key(body)

    get_psk_from_peers.assert_called_with(peer_id)


def test_rotate_pre_shared_key_missing_config():
    body = {
        "is_local_peer": True,
    }

    response = rotate_pre_shared_key(body)
    assert response.status_code == 400

    body = {
        "is_local_peer": False,
    }

    response = rotate_pre_shared_key(body)
    assert response.status_code == 400
