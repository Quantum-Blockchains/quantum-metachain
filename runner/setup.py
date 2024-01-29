import subprocess
import json
from os import path, mkdir, remove
import ed25519

ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")


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

config = {
    "__type__": "Config",
    "local_peer_id": peer_id,
    "local_server_port": enter_port("Enter local server port", 5001),
    "external_server_port": enter_port("Enter external server port", 5002),
    "node_http_rpc_port": enter_port("Enter node http rpc port server port", 9933),
    "psk_file_path": path.join(f'../.config/{peer_id}', "psk"),
    "psk_sig_file_path": path.join(f'../.config/{peer_id}', "psk_sig"),
    "node_key_file_path": path.join(f'../.config/{peer_id}', "node_key"),
    "key_rotation_time": key_rotation_time,
    "qrng_api_key": qrng_api_key,
    "recovery_check_interval": recovery_check_interval,
    "peers": {
    }
}

if not path.exists(node_config_dir):
    mkdir(node_config_dir)

node_key_file_path = path.join(node_config_dir, "node_key")
open(node_key_file_path, "wb").write(signing_key.to_ascii(encoding="hex"))

config_file_path = path.join(node_config_dir, "config.json")
json_object = json.dumps(config, indent=4)

with open(config_file_path, "w") as outfile:
    outfile.write(json_object)
