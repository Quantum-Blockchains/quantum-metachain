from local_server import rotate_pre_shared_key
from config import config
import node
from node import Node, NodeService
from os import path
from unittest.mock import MagicMock, patch


def test_rotate_pre_shared_key_local_peer_success():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()

    body = {
        "is_local_peer": True,
        "peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    }

    with patch("local_server.fetch_from_qrng") as fetch_from_qrng:
        fetch_from_qrng.return_value = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a"
        rotate_pre_shared_key(body)
        fetch_from_qrng.assert_called()

    # since we restart node in handler we can check not for psk but for signature

    assert path.exists(config.abs_psk_sig_file_path())


def test_rotate_pre_shared_key_not_local_peer_success():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"

    body = {
        "is_local_peer": False,
        "peer_id": peer_id
    }

    with patch("local_server.fetch_from_peers") as fetch_from_peers:
        fetch_from_peers.return_value = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a"
        rotate_pre_shared_key(body)
        fetch_from_peers.assert_called_with(peer_id)


def test_rotate_pre_shared_key_missing_config():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()

    # TODO make sure this is proper behaviour - if we provide just that peer is local do we really need peer_id?
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


