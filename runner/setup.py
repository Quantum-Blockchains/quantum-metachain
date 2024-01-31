import subprocess
import json
from os import path, mkdir, remove
import ed25519
from common.logger import log
import requests

ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")

config = {}


def enter_port(message, default_port):
    while True:
        try:
            port_str = input(f'{message} (Default port number is {default_port}): ')
            if port_str == "":
                return default_port
            port = int(port_str)
            if 1 <= port <= 65535:
                return port
            else:
                raise ValueError
        except ValueError:
            print('Error: this is NOT a VALID port number. The port must be a number from 1 to 65535.')
            continue


def data_exchange_eith_the_selected_node(address):
    return {
        "qkd": {
            "provider": "etsi014",
            "url": "",
            "client_cert_path": "",
            "cert_key_path": ""
        },
        "server_addr": address
    }


print("Create a configuration for a runner node.")

while True:
    try:
        tmp = int(input("Node key. If you already have a node key, enter 1 if you don't hae one, enter2: "))
        if tmp == 1 or tmp == 2:
            break
        else:
            raise ValueError
    except ValueError:
        print('ERROR: an option that does not exist has been selected. Only 1 or 2 can be selected.')
        continue

config_dir = path.join(ROOT_DIR, ".config")
if not path.exists(config_dir):
    mkdir(config_dir)

match tmp:
    case 1:
        str_tmp = input("Enter node key (64 hex characters): ")
        # TODO JEQB-269/Check enter node key
        open(path.join(config_dir, "tmp_key"), "w").write(str_tmp)
    case 2:
        signing_key, _ = ed25519.create_keypair()
        open(path.join(config_dir, "tmp_key"), "wb").write(signing_key.to_ascii(encoding="hex"))

args = "target/release/qmc-node" + " key" + " inspect-node-key" + " --file " + path.join(config_dir, "tmp_key")
peer_id = subprocess.check_output(args, shell=True, executable="/bin/bash", stderr=subprocess.STDOUT)
peer_id = peer_id.decode('utf-8').strip()

remove(path.join(config_dir, "tmp_key"))

node_config_dir = path.join(config_dir, peer_id)

while True:
    try:
        key_rotation_time = int(input("Enter key rotation time: "))
        if key_rotation_time > 0:
            break
        else:
            raise ValueError
    except ValueError:
        print('ERROR: enter the wrong value. Enter a number.')
        continue

qrng_api_key = input("Enter qrng api key: ")
# TODO JEQB-270/Check entered qrng api key

while True:
    try:
        recovery_check_interval = int(input("Enter recovery check interval: "))
        if key_rotation_time > 0:
            break
        else:
            raise ValueError
    except ValueError:
        print('ERROR: enter the wrong value. Enter a number.')
        continue

config["__type__"] = "Config"
config["local_peer_id"] = peer_id
config["local_server_port"] = enter_port("Enter local server port", 5001)
config["external_server_port"] = enter_port("Enter external server port", 5002),
config["node_http_rpc_port"] = enter_port("Enter node http rpc port server port", 9933)
config["psk_file_path"] = path.join(f'../.config/{peer_id}', "psk")
config["psk_sig_file_path"] = path.join(f'../.config/{peer_id}', "psk_sig")
config["node_key_file_path"] = path.join(f'../.config/{peer_id}', "node_key")
config["key_rotation_time"] = key_rotation_time
config["qrng_api_key"] = qrng_api_key
config["recovery_check_interval"] = recovery_check_interval
config["peers"] = {}

# TODO tut nado sdzielac request do jakiegos istnioncego noda zebys on podal do ktorych nodow trzeba poloczyc sie
# TODO sdzielac prowierku <peer_id>
peers = []
while True:
    try:
        list_of_peers = input("Enter peers (<peer_id>, <peer_id>, ...): ")
        peers = list_of_peers.split(", ")
        break
    except ValueError:
        print('ERROR: enter the wrong value. ')
        continue

# TODO sdzielac prowierku
addr = input("Enter one external_server address: ")
addresses = {}
addresses[addr] = False


for peer in peers:
    qwerty = True
    while qwerty:
        for a in list(addresses.keys()):
            if addresses[a]:
                continue
            search_peer_url = f"{a}/search_node/{peer}"
            print(f"Send request: {search_peer_url}")
            search_peer_response = requests.get(search_peer_url)
            if search_peer_response.status_code != 200:
                log.error(f"ERROR {search_peer_url}. Message: {search_peer_response.json()['message']}")
            else:
                response_body = search_peer_response.json()
                if response_body["found"]:
                    peer_info = data_exchange_eith_the_selected_node(response_body["external_server_address"])
                    config["peers"][peer] = peer_info
                    print(peer_info)
                    qwerty = False
                    break
                else:
                    addresses[a] = True
                    for x in response_body["peers"]:
                        if x in addresses.keys():
                            continue
                        else:
                            addresses[x] = False

if not path.exists(node_config_dir):
    mkdir(node_config_dir)

node_key_file_path = path.join(node_config_dir, "node_key")
open(node_key_file_path, "wb").write(signing_key.to_ascii(encoding="hex"))

config_file_path = path.join(node_config_dir, "config.json")
json_object = json.dumps(config, indent=4)

with open(config_file_path, "w") as outfile:
    outfile.write(json_object)
