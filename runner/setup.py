import subprocess
import json
from os import path, mkdir, remove
import ed25519
from common.logger import log
import requests
import argparse
import pathlib
from urllib.parse import urlparse
import base58


ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")


def url_type(arg):
    url = urlparse(arg)
    if all((url.scheme, url.netloc)):
        return arg
    raise argparse.ArgumentTypeError('Invalid URL')


def qrng_type(arg):
    qrng_key = arg.split('-')
    if qrng_key.__len__() != 5:
        raise argparse.ArgumentTypeError('Invalid QRNG api key')
    elif (qrng_key[0].__len__() != 8 or qrng_key[1].__len__() != 4 or qrng_key[2].__len__() != 4 or
          qrng_key[3].__len__() != 4 or qrng_key[4].__len__() != 12):
        raise argparse.ArgumentTypeError('Invalid QRNG api key')
    return arg


def peer_type(arg):
    if arg.__len__() != 52:
        raise argparse.ArgumentTypeError('Invalid peer')
    try:
        decoded = base58.b58decode(arg)
        return arg
    except base58.base58.InvalidBase58Error:
        raise argparse.ArgumentTypeError('Invalid peer')


def generate_node_key():
    config_dir = path.join(ROOT_DIR, ".config")
    if not path.exists(config_dir):
        mkdir(config_dir)
    signing_key, _ = ed25519.create_keypair()
    signing_key_hex = signing_key.to_ascii(encoding="hex")
    open(path.join(config_dir, ".tmp_key"), "wb").write(signing_key_hex)

    args_sub = "target/release/qmc-node" + " key" + " inspect-node-key" + " --file " + path.join(config_dir, ".tmp_key")
    peer_id = subprocess.check_output(args_sub, shell=True, executable="/bin/bash", stderr=subprocess.STDOUT)
    peer_id = peer_id.decode('utf-8').strip()

    node_config_dir = path.join(config_dir, peer_id)
    remove(path.join(config_dir, ".tmp_key"))
    if not path.exists(node_config_dir):
        mkdir(node_config_dir)
    node_key_file_path = path.join(node_config_dir, "node_key")
    open(node_key_file_path, "wb").write(signing_key_hex)

    print("Path to key file: ", node_key_file_path)
    print("PeerID:", peer_id)


def generate_config_node(args):
    config_dir = path.join(ROOT_DIR, ".config")
    node_config_dir = path.join(config_dir, args.peer_id)
    if not path.exists(node_config_dir):
        print("First of all, generate a key for the node using the command generate_node_key.")

    config = {
        "__type__": "Config",
        "local_peer_id": args.peer_id,
        "local_server_port": args.local_server_port,
        "external_server_port": args.external_server_port,
        "node_http_rpc_port": args.node_http_rpc_port,
        "psk_file_path": path.join(f'../.config/{args.peer_id}', "psk"),
        "psk_sig_file_path": path.join(f'../.config/{args.peer_id}', "psk_sig"),
        "node_key_file_path": path.join(f'../.config/{args.peer_id}', "node_key"),
        "node_logs_path": "",
        "key_rotation_time": args.key_rotation_time,
        "qrng_api_key": args.qrng_api_key,
        "recovery_check_interval": args.recovery_check_interval,
        "peers": {}
    }

    config_file_path = path.join(node_config_dir, "config.json")
    json_object = json.dumps(config, indent=4)

    with open(config_file_path, "w") as outfile:
        outfile.write(json_object)

    print("Path to config file: ", config_file_path)


def data_exchange_eith_the_selected_node(address):
    # TODO Make a real data exchange
    return {
        "qkd": {
            "provider": "etsi014",
            "url": "",
            "client_cert_path": "",
            "cert_key_path": ""
        },
        "server_addr": address
    }


def get_peer(args):
    config_dir = path.join(ROOT_DIR, ".config")
    node_config_dir = path.join(config_dir, args.peer_id)
    if not path.exists(node_config_dir):
        print("First of all, generate a key for the node using the command generate_node_key.")
    config_file_path = path.join(node_config_dir, "config.json")
    if not path.exists(config_file_path):
        print("First of all, generate a config file for the node using the command generate_config_node.")
    url = f"{args.boot_url}/get_peers_for_node/{args.peer_id}"
    get_peer_response = requests.get(url)
    if get_peer_response.status_code != 200:
        print(f"ERROR {url}. Message: {get_peer_response.json()['message']}")
    else:
        response_body = get_peer_response.json()
        peers = response_body["peers"]
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
                    print(f"Send request: {search_peer_url}")
                    search_peer_response = requests.get(search_peer_url)
                    if search_peer_response.status_code != 200:
                        log.error(f"ERROR {search_peer_url}. Message: {search_peer_response.json()['message']}")
                    else:
                        response_body = search_peer_response.json()
                        if response_body["found"]:
                            peer_info = data_exchange_eith_the_selected_node(response_body["external_server_address"])
                            peers_for_config[peer] = peer_info
                            print(peer_info)
                            tmp = False
                            break
                        else:
                            addresses[a] = True
                            for x in response_body["peers"]:
                                if x in addresses.keys():
                                    continue
                                else:
                                    addresses[x] = False

        with open(config_file_path, "r") as f:
            config = json.loads(f.read(), object_hook=lambda obj: obj)
        config["peers"] = peers_for_config
        json_object = json.dumps(config, indent=4)

        with open(config_file_path, "w") as outfile:
            outfile.write(json_object)


parser = argparse.ArgumentParser()

subp = parser.add_subparsers(dest='command', help='Commands to run', required=True)

# generate node key command
generate_key_parser = subp.add_parser('generate_node_key', help='Generate node key')

# generate config node command
generate_config_parser = subp.add_parser('generate_config_node', help='Generate config node')
generate_config_parser.add_argument('--peer_id', dest='peer_id', default=50, type=peer_type, required=True,
                    help='PeerId.')
generate_config_parser.add_argument('--local-server-port', dest='local_server_port', default=5001, type=int, nargs='?',
                    help='Local server port.')

generate_config_parser.add_argument('--external-server-port', dest='external_server_port', default=5002,
                    type=int, nargs='?', help='External server port')

generate_config_parser.add_argument('--node-http-rpc-port', dest='node_http_rpc_port', default=9933, type=int, nargs='?',
                    help='Node http rpc port.')
generate_config_parser.add_argument('--key-rotation-time', dest='key_rotation_time', default=50, type=int, nargs='?',
                    help='Key rotation time.')

generate_config_parser.add_argument('--recovery-check-interval', dest='recovery_check_interval', default=50, type=int,
                    nargs='?', help='Recovery check interval.')

generate_config_parser.add_argument('--qrng-api-key', dest='qrng_api_key', default=50, type=qrng_type, required=True,
                    help='Qrng api key.')

# get peers
get_peer_parser = subp.add_parser('get_peer', help='Get peer information.')
get_peer_parser.add_argument('--peer_id', dest='peer_id', default=50, type=peer_type, required=True,
                    help='PeerId.')
get_peer_parser.add_argument('--url', dest='boot_url', type=url_type, required=True, help='Boot url.')

args = parser.parse_args()


match args.command:
    case 'generate_node_key':
        generate_node_key()
    case 'generate_config_node':
        generate_config_node(args)
    case 'get_peer':
        get_peer(args)
    case _:
        pass
