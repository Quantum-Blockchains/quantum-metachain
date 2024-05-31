from os import path, mkdir, makedirs, rmdir
import shutil
import re
import requests
from colorama import init

init()
from colorama import Fore
import subprocess
import validators
import common.config
import common.file
from common.crypto import generate_ed25519, generate_self_signed_cert
from requests.packages.urllib3.exceptions import InsecureRequestWarning
import base58
import configparser
import readline, glob
import socket
from urllib.parse import urlparse
import proxy


requests.packages.urllib3.disable_warnings(InsecureRequestWarning)

ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
CONFIG_DIR = path.abspath(path.dirname(__file__) + "/../.config")

CONFIG_PROXY_FILE = "configClient.conf"
CONFIG_FILE = "config.json"
NODE_KEY_FILE = "node_key"


def complete(text, state):
    return (glob.glob(text+'*')+[None])[state]


def uint_type(arg):
    try:
        i = int(arg)
        if i < 0:
            raise Exception("The number must be equal to or greater than 0.")
    except Exception as err:
        raise Exception("The number must be equal to or greater than 0.")
    return i


def port_type(arg):
    try:
        i = int(arg)
        if (not i > 0) or (not i < 2 ** 16):
            raise Exception("Port numbers must be integers between 0 and 2**16")
    except Exception as err:
        raise Exception("Port numbers must be integers between 0 and 2**16")
    return i


def ip_type(arg):
    if not validators.ip_address.ipv4(arg):
        raise Exception('Invalid IP address')
    return arg


def url_type(arg):
    if not validators.url(arg):
        raise Exception('Invalid URL')
    return arg


def peer_type(arg):
    if arg.__len__() != 52:
        raise Exception('Invalid peer')
    try:
        decoded = base58.b58decode(arg)
        return arg
    except base58.base58.InvalidBase58Error:
        raise Exception('Invalid peer')


def editable_input(prompt, prefill='', default=None):
    readline.set_completer_delims(' \t\n;')
    readline.parse_and_bind("tab: complete")
    readline.set_completer(complete)
    readline.set_startup_hook(lambda: readline.insert_text(prefill))
    try:
        return input(prompt) or default
    finally:
        readline.set_startup_hook()


def enter_value(name_attr: str, message: str, type_val, default=None):
    old_value = None
    if hasattr(common.config.config_service.config, name_attr):
        old_value = getattr(common.config.config_service.config, name_attr)
    while True:
        if old_value is not None:
            value = editable_input(f"{message}: ", str(old_value), "")
        elif default is not None:
            value = editable_input(f"{message} (Default value {default}): ", "", default)
        else:
            value = editable_input(f"{message}: ", "", "")
        try:
            value = type_val(value)
            break
        except Exception as err:
            print(f"{Fore.RED}ERROR: {err}{Fore.RESET}")
    setattr(common.config.config_service.config, name_attr, value)


def enter_value_proxy(config: configparser.ConfigParser, item: str, message: str, type_val, default=None):
    old_value = None
    if item in config['settings']:
        old_value = config['settings'][item]
    while True:
        if old_value is not None:
            value = editable_input(f"{message}: ", str(old_value), "")
        elif default is not None:
            value = editable_input(f"{message} (Default value {Fore.GREEN}{default}{Fore.RESET}): ", "", default)
        else:
            value = editable_input(f"{message}: ", "", "")
        try:
            value = type_val(value)
            break
        except Exception as err:
            print(f"{Fore.RED}ERROR: {err}{Fore.RESET}")
    config['settings'][item] = str(value)
    return config


def enter_file(name_attr: str, message: str, to_node_dir: bool, node_dir: str, extension: str):
    old_path = None
    if hasattr(common.config.config_service.config, name_attr):
        attr = getattr(common.config.config_service.config, name_attr)
        if path.exists(path.join(node_dir, attr)):
            if path.join(node_dir, attr).endswith(extension):
                old_path = path.join(node_dir, attr)
    while True:
        if old_path is not None:
            path_file = editable_input(f"{message}: ", old_path, "")
        else:
            path_file = editable_input(f"{message}: ", "", "")
        if not path_file.endswith(extension):
            print(f"{Fore.RED}ERROR. The file extension must be {extension}.{Fore.RESET}")
            continue
        if path.exists(path_file) or path.exists(path_file):
            if to_node_dir and path_file != old_path:
                path_file = shutil.copy2(path_file, node_dir)
            tab = path_file.split("/")
            setattr(common.config.config_service.config, name_attr, tab[tab.__len__() - 1])
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

    # send target
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


    # body = {
    #     "peer_id": config.local_peer_id,
    #     "qkd_name": config.local_qkd_name,
    #     "port": config.proxy_external_server_port
    # }


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


try:
    i = 1
    # Name node
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    while True:
        node_name = editable_input("Enter the name you want to call the node: ", "", "")
        if node_name == "":
            print(f"{Fore.RED}The name of node cannot be an empty string.{Fore.RESET}")
            continue
        break

    # Node dir
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    node_dir = (str(input(f'Enter the path to the folder where the necessary data for node operation will be '
                          f'stored (Default - {path.join(CONFIG_DIR, node_name)}): ')) or
                str(path.join(CONFIG_DIR, node_name)))
    if not path.exists(node_dir):
        makedirs(node_dir)
    if path.exists(path.join(node_dir, CONFIG_FILE)):
        print("A configuration file already exists in this folder.")
        while True:
            tmp = str(
                input(f'Whether you want to change the data (1) or completely re-create the configuration (2) '
                      f'(Enter 1 or 2. Default 1): ')) or "1"
            if tmp == "1":
                common.config.init_config(path.join(node_dir, CONFIG_FILE))
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
    config_manager = common.file.FileManager(path.join(node_dir, CONFIG_FILE))
    common.config.config_service.config.local_node_name = node_name
    config_manager.create(common.config.config_service.config.to_json())

    # Node key
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
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
            tmp = input(f"Generate a new key (1), insert path key (2) or insert key (3) (Default 1): ") or "1"
            match tmp:
                case "1":
                    path_key, peer_id = enter_key(True, path.join(node_dir, NODE_KEY_FILE))
                    break
                case "2":
                    while True:
                        path_key = input("Enter path to the key: ")
                        try:
                            path_key, peer_id = enter_key(False, path_key)
                            break
                        except Exception as err:
                            print(f"Error: {err}")
                    break
                case "3":
                    while True:
                        key = input("Enter key: ")
                        if key.__len__() != 64:
                            print(f'ERROR: Invalid length of key: {key.__len__()}')
                        else:
                            path_key = path.join(node_dir, NODE_KEY_FILE)
                            open(path_key, "w").write(key)
                            try:
                                path_key, peer_id = enter_key(False, path_key)
                                break
                            except Exception as err:
                                print(f"Error: {err}")
                    break
                case _:
                    print(f'ERROR: Inadmissible value: {tmp}')
        common.config.config_service.config.local_peer_id = peer_id
        config_manager.create(common.config.config_service.config.to_json())

    # QKD name
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("local_qkd_name", "Enter the pQKD master SAE_ID", str, None)
    config_manager.create(common.config.config_service.config.to_json())

    # QKD url
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("local_qkd_url", "Enter the pQKD address", url_type, None)
    config_manager.create(common.config.config_service.config.to_json())

    # QKD notification port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("qkd_notification_port", "Enter the pQKD notification port", port_type, 8083)
    config_manager.create(common.config.config_service.config.to_json())

    # QKD secure port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("qkd_secure_port", "Enter the pQKD secure port", port_type, 8084)
    config_manager.create(common.config.config_service.config.to_json())

    # QKD cert and QKD key
    # tmp = common.config.config_service.config.local_qkd_url.split(":")
    # if tmp[0] == "https":
    #     if not path.exists(path.join(node_dir, "pqkd")):
    #         mkdir(path.join(node_dir, "pqkd"))
    #     enter_file("path_to_cert_pqkd", "Enter the path to pQKD cert path", True,
    #                path.join(node_dir, "pqkd"), ".crt")
    #     config_manager.create(common.config.config_service.config.to_json())
    #     enter_file("path_to_key_pqkd", "Enter the path to pQKD key path", True,
    #                path.join(node_dir, "pqkd"), ".key")
    #     config_manager.create(common.config.config_service.config.to_json())
    # else:
    #     common.config.config_service.config.path_to_cert_pqkd = ""
    #     common.config.config_service.config.path_to_key_pqkd = ""

    url_pqkd = urlparse(common.config.config_service.config.local_qkd_url)

    # QRNG address
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("qrng_url", "Enter the QRNG address", url_type,
                f'{url_pqkd.scheme}://{url_pqkd.hostname}:8085/qrng')
    config_manager.create(common.config.config_service.config.to_json())

    # QKD target
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    if not path.exists(path.join(node_dir, "pqkd")):
        makedirs(path.join(node_dir, "pqkd"))
    enter_file("local_qkd_target", "Enter the path to pQKD target", True,
               path.join(node_dir, "pqkd"), ".target")
    config_manager.create(common.config.config_service.config.to_json())

    # Public ip
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("public_ip", "Enter the local public ip", ip_type, None)
    # print(socket.gethostbyname(socket.gethostname()))
    # common.config.config_service.config.public_ip = socket.gethostbyname(socket.gethostname())
    config_manager.create(common.config.config_service.config.to_json())

    # local-server-port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("local_server_port", "Enter port for local server", port_type, "5001")
    config_manager.create(common.config.config_service.config.to_json())

    # external-server-port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("external_server_port", "Enter port for external server", port_type, "5002")
    config_manager.create(common.config.config_service.config.to_json())

    # proxy-external-server-port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("proxy_external_server_port", "Enter proxy port for external server", port_type, "6002")
    config_manager.create(common.config.config_service.config.to_json())

    # generate cert and key for runner
    # i += 1
    # print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    # print("Generate certificate for external server.")
    # country_name = editable_input("\tCOUNTRY NAME: ", "US", "US")
    # state_name = editable_input("\tSTATE OR PROVINCE NAME: ", "California", "California")
    # locality_name = editable_input("\tLOCALITY NAME: ", "San Francisco", "San Francisco")
    # organization_name = editable_input("\tORGANIZATION NAME: ", "My Company", "My Company")
    # common_name = editable_input("\tCOMMON NAME: ", "My node", "My node")
    # cert_pem, key_pem = generate_self_signed_cert(country_name, state_name, locality_name, organization_name,
    #                                               common_name, common.config.config_service.config.public_ip)
    # if not path.exists(f"{common.config.config_service.config.node_dir}/certificates/external"):
    #     makedirs(f"{common.config.config_service.config.node_dir}/certificates/external")
    # with open(f"{common.config.config_service.config.node_dir}/certificates/external/cert.pem", "wb") as f:
    #     f.write(cert_pem)
    # with open(f"{common.config.config_service.config.node_dir}/certificates/external/key.pem", "wb") as f:
    #     f.write(key_pem)
    # common.config.config_service.config.external_key = f"{common.config.config_service.config.node_dir}/certificates/external/key.pem"
    # common.config.config_service.config.external_cert = f"{common.config.config_service.config.node_dir}/certificates/external/cert.pem"

    # chain
    # i += 1
    # print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    # enter_file("chain", "Enter the path to chain specification", False,
    #            node_dir, ".json")
    # config_manager.create(common.config.config_service.config.to_json())

    # p2p-port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("p2p_port", "Enter port for p2p", port_type, "30333")
    config_manager.create(common.config.config_service.config.to_json())

    # node-http-rpc-port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("node_http_rpc_port", "Enter node http rpc port", port_type, "9933")
    config_manager.create(common.config.config_service.config.to_json())

    # key-rotation-time
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("key_rotation_time", "Enter key rotation time", uint_type, "50")
    config_manager.create(common.config.config_service.config.to_json())

    # recovery-check-interval
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    enter_value("recovery_check_interval", "Enter recovery check interval", uint_type, "50")
    config_manager.create(common.config.config_service.config.to_json())

    # psk_file_path, psk_sig_file_path, node_logs_path, runner_logs_path, peers
    # common.config.config_service.config.psk_file_path = path.join(node_dir, "psk")
    # common.config.config_service.config.psk_sig_file_path = path.join(node_dir, "psk_sig")
    # if not path.exists(path.join(node_dir, "logs")):
    #     mkdir(path.join(node_dir, "logs"))
    # common.config.config_service.config.node_logs_path = path.join(node_dir, "logs/node.log")
    # common.config.config_service.config.runner_logs_path = path.join(node_dir, "logs/runner.log")
    # common.config.config_service.config.peers = {}
    # config_manager.create(common.config.config_service.config.to_json())


    server_proxy_host = "31.182.67.107"
    server_proxy_port = "9998"
    client_proxy_id = "Node2"
    client_proxy_host = "192.168.8.106"
    client_proxy_listening = "5003:Node1"
    client_proxy_endpoint = "5003;192.168.8.106:5002"
    server_service_host = "31.182.67.107"
    server_service_port = "9991"

    path_to_config_proxy = path.join(node_dir, CONFIG_PROXY_FILE)

    if path.exists(path_to_config_proxy):
        config_proxy = configparser.ConfigParser()
        config_proxy.read(path_to_config_proxy)
    else:
        config_proxy = configparser.ConfigParser()
        config_proxy['settings'] = {}

    # config.type
    config_proxy['settings']['config.type'] = 'client'

    # server.proxy.host
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy,"server.proxy.host", "Enter server proxy host", ip_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    # server.proxy.port
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy, "server.proxy.port", "Enter server proxy port", port_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    # client.proxy.id
    config_proxy['settings']['client.proxy.id'] = common.config.config_service.config.local_peer_id

    # client.proxy.host
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy, "client.proxy.host", "Enter client proxy host", ip_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    # client.proxy.listening
    if 'client.proxy.listening' not in config_proxy['settings']:
        config_proxy['settings']['client.proxy.listening'] = ""

    # client.proxy.endpoint
    # port for external server
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy, "client.proxy.host", "Enter port for proxy external server", ip_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    endpoint_external_server = f'external_server;127.0.0.1:{common.config.config_service.config.external_server_port}'
    endpoint_kme = f'kme;{url_pqkd.netloc}'
    endpoint_notification = f'notification;{url_pqkd.hostname}:{common.config.config_service.config.qkd_notification_port}'
    endpoint_qkd = f'qkd;{url_pqkd.hostname}:{common.config.config_service.config.qkd_secure_port}'

    config_proxy['settings']['client.proxy.endpoint'] = f'{endpoint_external_server}, {endpoint_kme}, {endpoint_notification}, {endpoint_qkd}'

    # server.service.host
    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy, "server.service.host", "Enter server service host", ip_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    # server.service.port
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    config_proxy = enter_value_proxy(config_proxy, "server.service.port", "Enter server service port", port_type, None)
    with open(path.join(node_dir, CONFIG_PROXY_FILE), 'w') as configfile:
        config_proxy.write(configfile)

    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    print(f"Now the admin of the network you want connect to nust add your peer id "
          f"{common.config.config_service.config.local_peer_id} to the hupercube network.")

    proxy.proxy_service = proxy.Proxy(path.join(node_dir, CONFIG_PROXY_FILE))
    proxy.proxy_service.start()

    i += 1
    print(f"{Fore.GREEN}<Step {i}> {Fore.RESET}", end="")
    while True:
        # boot_url = str(input("Enter address of external server one node (PEER_ID): "))
        boot_peer_id = str(input("Enter peer id of one node (PEER_ID): "))
        boot_url = proxy.proxy_service.add_listning(boot_peer_id, "external_server")

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
