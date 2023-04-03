from unittest import mock
from unittest.mock import patch

import pytest

import node
from node import Node, NodeService, NodeTest
from web.local_server import rotate_pre_shared_key
from common.crypto import base58_to_hex, base64_to_hex, sign, verify, to_public, to_public_from_peerid


psk = "c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a"
block_number = 5
signature = sign(f'{block_number}{psk}', "df432c8e967aa21fdd287d3ea61fa85640a8309577f65b4ea78d49d514661654").hex()
# signature = "fe2550d4baf581af5f9cc9428e425b093fd4777fdfeb7d00a9b52261754a56ec5034dd0fe3570d9d7f7a21b9d2d2007cba1afe773430dbd79b7c0cf37a55e803"


@pytest.fixture()
def before_each():
    node.node_service = node.node_service = NodeService(NodeTest())
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
        "peer_id": "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW",
        "block_num": block_number
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
        "peer_id": peer_id,
        "block_num": block_number
    }
    rotate_pre_shared_key(body)

    get_psk_from_peers.assert_called_with(block_number, peer_id)


@patch("core.pre_shared_key.get_psk_from_peers")
def test_rotate_pre_shared_key_when_other_peer_is_chosen_but_returns_none_for_the_first_time(get_psk_from_peers, before_each):
    get_psk_from_peers.side_effect = [None, ("c7ce4948991367f8f08c473f1bdf3a45945951eb4038f735a76e840d36c27b1a",
                                             "17d1dc882d5ed8346be27a2529d046afe42b56825e374236ae0a80ad448086027e2b2982a2eb8f38221cf3aebc223c01b332101b1c7e5718651d076b430e9100")]
    peer_id = "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW"
    body = {
        "is_local_peer": False,
        "peer_id": peer_id,
        "block_num": block_number
    }
    rotate_pre_shared_key(body)

    get_psk_from_peers.assert_called_with(block_number, peer_id)

    # Assert if mock was called twice
    assert get_psk_from_peers.call_args_list == [
        mock.call(block_number, peer_id),
        mock.call(block_number, peer_id),
    ]


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
