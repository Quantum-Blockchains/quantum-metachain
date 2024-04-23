import requests
from common.logger import log
from os import path
import json


ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")


def data_exchange_eith_the_selected_node(url, local_peer_id, local_qkd_url, external_address, local_qkd_name):
    # TODO Make a real data exchange
    body = {
        "peer_id": local_peer_id,
        "qkd_name": local_qkd_name,
        "server_addr": external_address
    }
    response = requests.post(url+"/data_qkd_exchange", json=body)
    if response.status_code != 200:
        log.error(f"ERROR {url}. Message: {response.json()['message']}")
    else:
        response_body = response.json()
        return {
            "qkd": {
                "provider": "etsi014",
                "url": local_qkd_url + "/api/v1/keys/" + response_body["qkd_name"],
                "client_cert_path": "../certificates/qbck-client.crt",
                "cert_key_path": "../certificates/qbck-client.key"
            },
            "server_addr": response_body["server_addr"],
        }


def get_peer(args):
    # config_dir = path.join(ROOT_DIR, ".config")
    # node_config_dir = path.join(config_dir, args.peer_id)
    # if not path.exists(node_config_dir):
    #     print("First of all, generate a key for the node using the command generate_node_key.")
    if not path.exists(args.config_file):
        print("Configuration file not found")
    else:
        with open(args.config_file, "r") as f:
            config = json.loads(f.read(), object_hook=lambda obj: obj)
        # config_file_path = path.join(node_config_dir, "config.json")
        # hostname = socket.gethostname()
        # IPAddr = socket.gethostbyname(hostname)

    # if not path.exists(config_file_path):
    #     print("First of all, generate a config file for the node using the command generate_config_node.")
        url = f"{args.boot_url}/get_peers_for_node/{config['local_peer_id']}"
        get_peer_response = requests.get(url)
        if get_peer_response.status_code != 200:
            print(f"ERROR {url}. Message: {get_peer_response.json()['message']}")
        else:
            response_body = get_peer_response.json()
            peers = response_body["peers"]
            print(f'Peers to be contacted: {peers}')
            peers_for_config = {}
            addresses = {}
            addresses[args.boot_url] = False

            for peer in peers:
                tmp = True
                while tmp:
                    for a in list(addresses.keys()):
                        if addresses[a]:
                            continue
                        search_peer_url = f"{a}/search_node/{peer}"
                        # print(f"Send request: {search_peer_url}")
                        search_peer_response = requests.get(search_peer_url)
                        if search_peer_response.status_code != 200:
                            log.error(f"ERROR {search_peer_url}. Message: {search_peer_response.json()['message']}")
                        else:
                            response_body = search_peer_response.json()
                            if response_body["found"]:
                                peer_info = data_exchange_eith_the_selected_node(
                                    response_body["external_server_address"], config["local_peer_id"], config["local_qkd_url"],
                                    "http://"+config["public_ip"]+':'+str(config["external_server_port"]), config["local_qkd_name"])
                                peers_for_config[peer] = peer_info
                                print(f'Data exchange from the peer {peer} has been successfully completed.')
                                tmp = False
                                break
                            else:
                                addresses[a] = True
                                for x in response_body["peers"]:
                                    if x in addresses.keys():
                                        continue
                                    else:
                                        addresses[x] = False

            config["peers"] = peers_for_config
            json_object = json.dumps(config, indent=4)

            with open(args.config_file, "w") as outfile:
                outfile.write(json_object)

