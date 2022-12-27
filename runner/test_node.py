
import time
import requests
import sys
from config import Config

peers = [
    'QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdB',
    'QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV'
]

config_path = sys.argv[1]
config = Config(config_path)

def send_psk_rotation_request(runner_port, peer_index):
    url = "http://localhost:{port}/psk".format(port = runner_port)
    if peers[peer_index] == config.config['local_peer_id']:
        is_local_peer = True
    else:
        is_local_peer = False
    data = {'peer_id': peers[peer_index], 'is_local_peer': is_local_peer}
    requests.post(url, json=data)


period = time.localtime().tm_min + 1
while True:
    if time.localtime().tm_sec == 0 and time.localtime().tm_min == period:
        send_psk_rotation_request(config.config['local_server_port'], time.localtime().tm_min % 2)
        if period == 59:
            period = 0
        else:
            period += 1
