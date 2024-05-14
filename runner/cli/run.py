from threading import Thread

import node
import common.config
import common.file
from common.config import create_node_info_dir
from node import Node, NodeService
from common.logger import log, add_logs_handler_file
from core import pre_shared_key
from web.local_server import LocalServerWrapper
from web.external_server import ExternalServerWrapper
from time import sleep
import requests
from os import path

ROOT_DIR = path.abspath(path.dirname(__file__) + "/../..")


def run(args):
    common.config.init_config(args.config_file)
    common.file.initialise_file_managers(args.config_file)

    create_node_info_dir()
    add_logs_handler_file()
    substrate_args = [path.join(ROOT_DIR, "target/release/qmc-node"),
                      "--name", common.config.config_service.config.local_node_name,
                      f"--base-path=/tmp/{common.config.config_service.config.local_node_name}",
                      f"--chain={common.config.config_service.config.chain}",
                      "--port", str(common.config.config_service.config.p2p_port),
                      "--public-addr", f"/ip4/{common.config.config_service.config.public_ip}/tcp/"
                      f"{common.config.config_service.config.p2p_port}",
                      "--rpc-port", str(common.config.config_service.config.node_http_rpc_port),
                      "--psk-file", common.config.config_service.config.psk_file_path,
                      "--runner-port", str(common.config.config_service.config.local_server_port),
                      "--node-key-file", common.config.config_service.config.node_key_file_path]
    for arg in args.startup_args:
        substrate_args.append(arg)
    node.node_service = NodeService(Node(substrate_args))

    try:
        log.info("Starting QMC runner...")
        if not common.file.psk_file_manager.exists():
            psk_obj = None
            while psk_obj is None:
                psk_obj = pre_shared_key.get_psk_from_peers()
                sleep(10)
            common.file.psk_file_manager.create(psk_obj.psk)
            common.file.psk_sig_file_manager.create(psk_obj.signature)

            block_for_start = None

            # TODO current_block < block_for_start

            peers = common.config.config_service.config.peers

            while block_for_start is None:
                for peer_id, peer_config in peers.items():
                    try:
                        response = requests.get(f"{peer_config['server_addr']}/get_number_block_for_restart", verify=False)
                        if response.status_code != 200:
                            log.warning(f"Failed to get the block number to start from peer {peer_id}")
                        else:
                            response_body = response.json()
                            if response_body["num_block_for_restart"] != 0:
                                block_for_start = response_body["num_block_for_restart"]
                                break
                    except Exception as err:
                        log.warning(f"Failed to get the block number to start from peer {peer_id}. Error: {str(err)}")

            log.info(f"Number of the block in which the node will start: {block_for_start}")

            tmp = True

            while tmp:
                for peer_id, peer_config in peers.items():
                    try:
                        response = requests.get(f"{peer_config['server_addr']}/get_current_number_block", verify=False)
                    except Exception as err:
                        log.error(f"Failed to get the current block number from peer {peer_id}. Error: {str(err)}")
                    if response.status_code != 200:
                        log.error(f"Failed to get the current block number from peer {peer_id}")
                        sleep(4)
                    else:
                        response_body = response.json()
                        log.info(
                            f"Current block: {response_body['current_block']}. The node will start in the block {block_for_start}")
                        if response_body["current_block"] >= block_for_start:
                            tmp = False
                            break
                        else:
                            sleep(4)

        node.node_service.current_node.start()

        external_server = ExternalServerWrapper()
        external_thread = Thread(target=external_server.run, args=())
        log.info("Starting external server...")
        external_thread.start()

        log.info("Starting local server...")
        local_server = LocalServerWrapper()
        local_server.run()

    except Exception as e:
        log.error(str(e))
    finally:
        log.info("Closing QMC processes...")
        node.node_service.current_node.terminate()