from web.local_server import rotate_pre_shared_key
import node
from node import Node, NodeService
from unittest.mock import patch


def test_rotate_pre_shared_key_local_peer_success():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()

    body = {
        "is_local_peer": True,
        "peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    }

    with patch("web.local_server.generate_psk_from_qrng") as generate_psk_from_qrng:
        generate_psk_from_qrng.return_value = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a"
        rotate_pre_shared_key(body)
        generate_psk_from_qrng.assert_called()


def test_rotate_pre_shared_key_not_local_peer_success():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"

    body = {
        "is_local_peer": False,
        "peer_id": peer_id
    }

    with patch("web.local_server.get_psk_from_peers") as get_psk_from_peers:
        get_psk_from_peers.return_value = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a", "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100"
        rotate_pre_shared_key(body)
        get_psk_from_peers.assert_called_with(peer_id)


def test_rotate_pre_shared_key_missing_config():
    node.node_service = NodeService(Node(["python3", "node_simulator.py"]))
    node.node_service.current_node.start()

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
