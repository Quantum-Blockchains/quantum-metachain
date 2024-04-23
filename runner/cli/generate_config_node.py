from os import path, mkdir
import json

ROOT_DIR = path.abspath(path.dirname(__file__) + "/../..")
CONFIG_DIR = path.abspath(path.dirname(__file__) + "/../../.config")


def create_node_dir(name_node, node_dir=None) -> str:
    if node_dir is not None:
        if path.exists(path.join(ROOT_DIR, node_dir)):
            return path.join(ROOT_DIR, node_dir)
        else:
            raise Exception(f"The folder {path.join(ROOT_DIR, node_dir)} does not exist")
    else:
        node_dir = path.join(CONFIG_DIR, name_node)
        return node_dir


def generate_config_node(args):
    # config_dir = path.join(ROOT_DIR, ".config")
    # node_config_dir = path.join(config_dir, args.peer_id)
    # if not path.exists(node_config_dir):
    #     print("First of all, generate a key for the node using the command generate_node_key.")
    try:
        node_dir = create_node_dir(args.node_name, args.node_dir)

        config = {
            "__type__": "Config",
            "local_node_name": args.node_name,
            "local_peer_id": "",
            "node_dir": node_dir,
            "local_qkd_url": args.qkd_url,
            "local_qkd_name": args.qkd_name,
            "public_ip": args.public_ip,
            "local_server_port": args.local_server_port,
            "external_server_port": args.external_server_port,
            "node_http_rpc_port": args.node_http_rpc_port,
            "psk_file_path": path.join(node_dir, "psk"),
            "psk_sig_file_path": path.join(node_dir, "psk_sig"),
            "node_key_file_path": path.join(node_dir, "node_key"),
            "node_logs_path": path.join(node_dir, 'logs/node.log'),
            "runner_logs_path": path.join(node_dir, 'logs/runner.log'),
            "key_rotation_time": args.key_rotation_time,
            "qrng_api_key": args.qrng_api_key,
            "recovery_check_interval": args.recovery_check_interval,
            "peers": {}
        }

        if not path.exists(CONFIG_DIR):
            mkdir(CONFIG_DIR)

        if not path.exists(node_dir):
            mkdir(node_dir)
        else:
            raise Exception(f"The folder for node {args.node_name} exist.")

        if not path.exists(path.join(node_dir, "logs")):
            mkdir(path.join(node_dir, "logs"))

        config_file_path = path.join(node_dir, "config.json")
        json_object = json.dumps(config, indent=4)

        with open(config_file_path, "w") as outfile:
            outfile.write(json_object)
        print("Path to node dir: ", node_dir)
        print("Path to config file: ", config_file_path)
    except Exception as e:
        print(f'Failed to create the configuration. {str(e)}')
