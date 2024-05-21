# import time
from os import path, mkdir, makedirs, rmdir
import shutil
import re
import requests
# from pynput.keyboard import Controller
from threading import Thread
from colorama import init
init()
from colorama import Fore
import subprocess
import validators
import common.config
import common.file
from cli.types import ip_type, url_type, qrng_type, peer_type, substrate_arguments, port_type, uint_type
from common.crypto import generate_ed25519, generate_self_signed_cert
from requests.packages.urllib3.exceptions import InsecureRequestWarning
import pyautogui


requests.packages.urllib3.disable_warnings(InsecureRequestWarning)

ROOT_DIR = path.abspath(path.dirname(__file__) + "/../..")
CONFIG_DIR = path.abspath(path.dirname(__file__) + "/../../.config")

def enter_value(name_attr: str, message: str, type_val, default=None):
    old_value = None
    if hasattr(common.config.config_service.config, name_attr):
        old_value = getattr(common.config.config_service.config, name_attr)
    while True:
        if old_value is not None:
            value = editable_input(f"{message}: ", str(old_value), "")
            # value = str(input(f"{message} (Default old value {old_value}): ")) or str(old_value)
        elif default is not None:
            # value = str(input(f"{message} (Default value {Fore.GREEN}{default}{Fore.RESET}): ")) or default
            value = editable_input(f"{message} (Default value {Fore.GREEN}{default}{Fore.RESET}): ", "", default)
        else:
            # value = str(input(f"{message}"))
            value = editable_input(f"{message}: ", "", "")
        try:
            value = type_val(value)
            break
        except Exception as err:
            print(f"{Fore.RED}ERROR: {err}{Fore.RESET}")
    setattr(common.config.config_service.config, name_attr, value)


def editable_input(message, value, default):
    # keyboard = Controller()
    print(message, end="")
    print(Fore.GREEN, end="")
    # Thread(target=keyboard.type, args=(value,)).start()
    Thread(target=pyautogui.write, args=(value,)).start()
    modified_input = input() or default
    print(Fore.RESET, end="")
    return modified_input


def enter_file(name_attr: str, message: str, to_node_dir: bool, node_dir: str, extension: str):
    old_path = None
    if hasattr(common.config.config_service.config, name_attr):
        if path.exists(getattr(common.config.config_service.config, name_attr)):
            if getattr(common.config.config_service.config, name_attr).endswith(extension):
                old_path = getattr(common.config.config_service.config, name_attr)
    while True:
        if old_path is not None:
            path_file = editable_input(f"{message}: ", old_path, "")
        else:
            path_file = editable_input(f"{message}: ", "", "")
        if not path_file.endswith(extension):
            print(f"{Fore.RED}ERROR. The file extension must be {extension}.{Fore.RESET}")
            continue
        if path.exists(path_file):
            if to_node_dir and path_file != old_path:
                path_file = shutil.copy2(path_file, node_dir)
            setattr(common.config.config_service.config, name_attr, path_file)
            break
        else:
            print(f"{Fore.RED}ERROR. File {path_file} does not exist.{Fore.RESET}")


def enter_key(new: bool, path_key: str):
    if new:
        sk_hex = generate_ed25519()
        open(path_key, "w").write(sk_hex)
        args_sub = (path.join(ROOT_DIR, "target/release/qmc-node") + " key" + " inspect-node-key" + " --file "
                    + path_key)
        peer_id = subprocess.check_output(args_sub, shell=True, executable="/bin/bash", stderr=subprocess.STDOUT)
        peer_id = peer_id.decode('utf-8').strip()
    else:
        if not path.exists(path_key):
            raise Exception(f"File {path_key} does not exist.")
        try:
            args_sub = (path.join(ROOT_DIR, "target/release/qmc-node") + " key" + " inspect-node-key" + " --file "
                        + path_key)
            peer_id = subprocess.check_output(args_sub, shell=True, executable="/bin/bash", stderr=subprocess.STDOUT)
        except Exception as err:
            raise Exception(f"The file {path_key} does not contain a hex key Ed25519.")
        peer_id = peer_id.decode('utf-8').strip()
    return path_key, peer_id


def data_exchange_with_the_selected_node(url, config):
    target = {"target": open(config.local_qkd_target, 'rb')}
    response_target = requests.post(url + "/targets_exchange", files=target, verify=False)
    if response_target.status_code != 200:
        print(f"ERROR {url}. Message: {response_target.json()['message']}")
    else:
        fname = re.findall("filename=(.+)", response_target.headers["Content-Disposition"])[0]
        if not path.exists(path.join(config.node_dir, 'targets')):
            mkdir(path.join(config.node_dir, 'targets'))
        with open(path.join(config.node_dir, f'targets/{fname}'), 'wb') as fw:
            fw.write(response_target.content)
    body = {
        "peer_id": config.local_peer_id,
        "qkd_name": config.local_qkd_name,
        "server_addr": "https://" + config.public_ip + ':' + str(config.external_server_port)
    }
    response = requests.post(url + "/data_qkd_exchange", json=body, verify=False)
    if response.status_code != 200:
        print(f"ERROR {url}. Message: {response.json()['message']}")
    else:
        response_body = response.json()
        return {
            "qkd": {
                "provider": "etsi014",
                "url": config.local_qkd_url + "/api/v1/keys/" + response_body["qkd_name"],
                "client_cert_path": config.path_to_cert_pqkd,
                "cert_key_path": config.path_to_key_pqkd
            },
            "server_addr": response_body["server_addr"],
        }


def generate_config_node_wizard():
    try:
        # Name node
        print(f"{Fore.GREEN}<Step 1> {Fore.RESET}", end="")
        while True:
            node_name = editable_input("Enter the name you want to call the node: ", "", "")
            if node_name == "":
                print(f"{Fore.RED}The name of node cannot be an empty string.{Fore.RESET}")
                continue
            break

        # Node dir
        print(f"{Fore.GREEN}<Step 2> {Fore.RESET}", end="")
        node_dir = (str(input(f'Enter the path to the folder where the necessary data for node operation will be '
                              f'stored (Default - {Fore.GREEN}{path.join(CONFIG_DIR, node_name)}{Fore.RESET}): ')) or
                    str(path.join(CONFIG_DIR, node_name)))
        if not path.exists(node_dir):
            makedirs(node_dir)
        if path.exists(path.join(node_dir, "config.json")):
            print("A configuration file already exists in this folder.")
            while True:
                tmp = str(
                    input(f'Whether you want to change the data (1) or completely re-create the configuration (2) '
                          f'(Enter 1 or 2. Default 1) ')) or "1"
                if tmp == "1":
                    common.config.init_config(path.join(node_dir, "config.json"))
                    break
                elif tmp == "2":
                    rmdir(node_dir)
                    makedirs(node_dir)
                    common.config.config_service = common.config.ConfigService(common.config.Config({}))
                    break
                else:
                    print(f'Inadmissible value: {tmp}')
        else:
            common.config.config_service = common.config.ConfigService(common.config.Config({}))
        config_manager = common.file.FileManager(path.join(node_dir, "config.json"))
        common.config.config_service.config.local_node_name = node_name
        common.config.config_service.config.node_dir = node_dir
        config_manager.create(common.config.config_service.config.to_json())

        # Node key
        print(f"{Fore.GREEN}<Step 3> {Fore.RESET}", end="")
        old_peer_id = None
        old_path_to_key = None
        if hasattr(common.config.config_service.config, 'local_peer_id'):
            old_peer_id = common.config.config_service.config.local_peer_id
        if hasattr(common.config.config_service.config, 'node_key_file_path'):
            old_path_to_key = common.config.config_service.config.node_key_file_path
        new_key = True
        if old_peer_id is not None:
            if old_path_to_key is not None:
                while True:
                    res = input(f"Leave the key {old_path_to_key} with peer id {old_peer_id} (yes/no): ") or "yes"
                    if res == "yes":
                        new_key = False
                        break
                    elif res == "no":
                        break
                    else:
                        print(f'Inadmissible value: {res}')
        if new_key:
            while True:
                tmp = input(f"Generate a new key (1) or insert key (2) (Default 1):") or "1"
                if tmp == "1":
                    path_key, peer_id = enter_key(True, path.join(node_dir, "node_key"))
                    break
                elif tmp == "2":
                    while True:
                        path_key = input("Enter path to the key: ")
                        try:
                            path_key, peer_id = enter_key(False, path_key)
                            break
                        except Exception as err:
                            print(f"Error: {err}")
                    break
                else:
                    print(f'ERROR: Inadmissible value: {tmp}')
            common.config.config_service.config.local_peer_id = peer_id
            common.config.config_service.config.node_key_file_path = path_key
            config_manager.create(common.config.config_service.config.to_json())

        # QKD url
        print(f"{Fore.GREEN}<Step 4> {Fore.RESET}", end="")
        enter_value("local_qkd_url", "Enter the pQKD address", url_type, None)
        config_manager.create(common.config.config_service.config.to_json())

        # QKD cert and QKD key
        if not path.exists(path.join(node_dir, "certificates")):
            mkdir(path.join(node_dir, "certificates"))
        if common.config.config_service.config.local_qkd_url[
           :common.config.config_service.config.local_qkd_url.find(":")] == "https":
            enter_file("path_to_cert_pqkd", "Enter the path to pQKD cert path", True,
                       path.join(node_dir, "certificates"), ".crt")
            config_manager.create(common.config.config_service.config.to_json())
            enter_file("path_to_key_pqkd", "Enter the path to pQKD key path", True,
                       path.join(node_dir, "certificates"), ".key")
            config_manager.create(common.config.config_service.config.to_json())
        else:
            common.config.config_service.config.path_to_cert_pqkd = ""
            common.config.config_service.config.path_to_key_pqkd = ""

        # QKD name
        print(f"{Fore.GREEN}<Step 5> {Fore.RESET}", end="")
        enter_value("local_qkd_name", "Enter the pQKD master SAE_ID", str, None)
        config_manager.create(common.config.config_service.config.to_json())

        # QKD target
        print(f"{Fore.GREEN}<Step 6> {Fore.RESET}", end="")
        enter_file("local_qkd_target", "Enter the path to pQKD target", True, node_dir,
                   ".target")
        config_manager.create(common.config.config_service.config.to_json())

        # QRNG address
        print(f"{Fore.GREEN}<Step 7> {Fore.RESET}", end="")
        enter_value("qrng_url", "Enter the QRNG address", url_type,
                    f"{common.config.config_service.config.local_qkd_url}/qrng")
        config_manager.create(common.config.config_service.config.to_json())

        # Public ip
        print(f"{Fore.GREEN}<Step 8> {Fore.RESET}", end="")
        enter_value("public_ip", "Enter the local public ip", ip_type, None)
        config_manager.create(common.config.config_service.config.to_json())

        # local-server-port
        print(f"{Fore.GREEN}<Step 9> {Fore.RESET}", end="")
        enter_value("local_server_port", "Enter port for local server", port_type, "5001")
        config_manager.create(common.config.config_service.config.to_json())

        # external-server-port
        print(f"{Fore.GREEN}<Step 10> {Fore.RESET}", end="")
        enter_value("external_server_port", "Enter port for external server", port_type, "5002")
        config_manager.create(common.config.config_service.config.to_json())

        # generate cert and key for runner
        print(f"{Fore.GREEN}<Step 12> {Fore.RESET}", end="")
        print("Generate certificate for external server.")
        country_name = editable_input("\tCOUNTRY NAME: ", "US", "US")
        state_name = editable_input("\tSTATE OR PROVINCE NAME: ", "California", "California")
        locality_name = editable_input("\tLOCALITY NAME: ", "San Francisco", "San Francisco")
        organization_name = editable_input("\tORGANIZATION NAME: ", "My Company", "My Company")
        common_name = editable_input("\tCOMMON NAME: ", "My node", "My node")
        cert_pem, key_pem = generate_self_signed_cert(country_name, state_name, locality_name, organization_name,
                                                      common_name, common.config.config_service.config.public_ip)
        if not path.exists(f"{common.config.config_service.config.node_dir}/certificates/external"):
            makedirs(f"{common.config.config_service.config.node_dir}/certificates/external")
        with open(f"{common.config.config_service.config.node_dir}/certificates/external/cert.pem", "wb") as f:
            f.write(cert_pem)
        with open(f"{common.config.config_service.config.node_dir}/certificates/external/key.pem", "wb") as f:
            f.write(key_pem)
        common.config.config_service.config.external_key = f"{common.config.config_service.config.node_dir}/certificates/external/key.pem"
        common.config.config_service.config.external_cert = f"{common.config.config_service.config.node_dir}/certificates/external/cert.pem"

        # chain
        print(f"{Fore.GREEN}<Step 11> {Fore.RESET}", end="")
        enter_file("chain", "Enter the path to chain specification", False,
                   node_dir, ".json")
        config_manager.create(common.config.config_service.config.to_json())

        # p2p-port
        print(f"{Fore.GREEN}<Step 12> {Fore.RESET}", end="")
        enter_value("p2p_port", "Enter port for p2p", port_type, "30333")
        config_manager.create(common.config.config_service.config.to_json())

        # node-http-rpc-port
        print(f"{Fore.GREEN}<Step 13> {Fore.RESET}", end="")
        enter_value("node_http_rpc_port", "Enter node http rpc port", port_type, "9933")
        config_manager.create(common.config.config_service.config.to_json())

        # key-rotation-time
        print(f"{Fore.GREEN}<Step 14> {Fore.RESET}", end="")
        enter_value("key_rotation_time", "Enter key rotation time", uint_type, "50")
        config_manager.create(common.config.config_service.config.to_json())

        # recovery-check-interval
        print(f"{Fore.GREEN}<Step 15> {Fore.RESET}", end="")
        enter_value("recovery_check_interval", "Enter recovery check interval", uint_type, "50")
        config_manager.create(common.config.config_service.config.to_json())

        # psk_file_path, psk_sig_file_path, node_logs_path, runner_logs_path, peers
        common.config.config_service.config.psk_file_path = path.join(node_dir, "psk")
        common.config.config_service.config.psk_sig_file_path = path.join(node_dir, "psk_sig")
        if not path.exists(path.join(node_dir, "logs")):
            mkdir(path.join(node_dir, "logs"))
        common.config.config_service.config.node_logs_path = path.join(node_dir, "logs/node.log")
        common.config.config_service.config.runner_logs_path = path.join(node_dir, "logs/runner.log")
        common.config.config_service.config.peers = {}
        config_manager.create(common.config.config_service.config.to_json())

        print(f"{Fore.GREEN}<Step 16> {Fore.RESET}", end="")
        print(f"Now the admin of the network you want connect to nust add your peer id "
              f"{common.config.config_service.config.local_peer_id} to the hupercube network.")

        print(f"{Fore.GREEN}<Step 17> {Fore.RESET}", end="")
        while True:
            boot_url = str(input("Enter address of external server one node: "))
            if not validators.url(boot_url):
                print(f'{Fore.RED}ERROR: Inadmissible value: {boot_url}{Fore.RESET}')
                continue
            url = f"{boot_url}/get_peers_for_node/{common.config.config_service.config.local_peer_id}"
            try:
                get_peer_response = requests.get(url, verify=False)
                if get_peer_response.status_code != 200:
                    print(f"{Fore.RED}ERROR {url}. Message: {get_peer_response.json()['message']}{Fore.RESET}")
                    continue
                else:
                    response_body = get_peer_response.json()
                    peers = response_body["peers"]
                    print(f'Peers to be contacted: {peers}')
                    break
            except Exception as err:
                print(f"{Fore.RED}ERROR. {err}{Fore.RESET}")
        peers_for_config = {}
        addresses = {boot_url: False}
        for peer in peers:
            tmp = True
            while tmp:
                for a in list(addresses.keys()):
                    if addresses[a]:
                        continue
                    search_peer_url = f"{a}/search_node/{peer}"
                    # print(f"Send request: {search_peer_url}")
                    try:
                        search_peer_response = requests.get(search_peer_url, verify=False)
                    except Exception as err:
                        print("ERROR: err")
                    if search_peer_response.status_code != 200:
                        print(f"ERROR {search_peer_url}. Message: {search_peer_response.json()['message']}")
                    else:
                        response_body = search_peer_response.json()
                        if response_body["found"]:
                            try:
                                peer_info = data_exchange_with_the_selected_node(
                                    response_body["external_server_address"], common.config.config_service.config)
                            except Exception as err:
                                print(f"ERROR: {err}")
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

        common.config.config_service.config.peers = peers_for_config
        config_manager.create(common.config.config_service.config.to_json())

        print(f"Node configuration is complete.")

    except KeyboardInterrupt as err:
        print(f"\nNode configuration is not complete.")
