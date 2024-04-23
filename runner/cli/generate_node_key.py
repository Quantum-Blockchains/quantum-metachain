from cryptography.hazmat.primitives.asymmetric import ed25519
from os import path, mkdir, remove
import subprocess
from cryptography.hazmat.primitives._serialization import PrivateFormat, Encoding
from cryptography.hazmat.primitives import serialization
import common.config
import common.file


ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")


def generate_node_key(args):
    try:
    # config_dir = path.join(ROOT_DIR, ".config")
        config_file = path.join(args.node_dir, "config.json")
        common.config.init_config(config_file)
        config_manager = common.file.FileManager(config_file)
        if not path.exists(args.node_dir):
            raise Exception("No dir")

        private_key = ed25519.Ed25519PrivateKey.generate()
        signing_key_hex = private_key.private_bytes(Encoding.Raw, PrivateFormat.Raw, serialization.NoEncryption()).hex()
        open(path.join(args.node_dir, ".tmp_key"), "w").write(signing_key_hex)

        args_sub = "target/release/qmc-node" + " key" + " inspect-node-key" + " --file " + path.join(args.node_dir, ".tmp_key")
        peer_id = subprocess.check_output(args_sub, shell=True, executable="/bin/bash", stderr=subprocess.STDOUT)
        peer_id = peer_id.decode('utf-8').strip()

        common.config.config_service.config.local_peer_id = peer_id
        config_manager.create(common.config.config_service.config.to_json())
        # node_config_dir = path.join(args.node_dir, peer_id)
        remove(path.join(args.node_dir, ".tmp_key"))
        # if not path.exists(node_config_dir):
        #     mkdir(node_config_dir)
        node_key_file_path = path.join(args.node_dir, "node_key")
        open(node_key_file_path, "w").write(signing_key_hex)

        print("Path to key file: ", node_key_file_path)
        print("PeerID:", peer_id)
    except Exception as err:
        print(str(err))