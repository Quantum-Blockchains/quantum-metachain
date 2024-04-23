import argparse
from cli.types import ip_type, url_type, qrng_type, peer_type, substrate_arguments
import pathlib


parser = argparse.ArgumentParser()

subp = parser.add_subparsers(dest='command', help='Commands to run', required=True)

# generate node key command
generate_key_parser = subp.add_parser('generate-node-key', help='Generate node key')
generate_key_parser.add_argument('--node-dir', dest='node_dir', type=str, required=True,
                    help='Substrate node name. (Default none)')

# generate config node command
generate_config_parser = subp.add_parser('generate-config-node', help='Generate config node')
# generate_config_parser.add_argument('--peer_id', dest='peer_id', type=peer_type, required=True,
#                     help='PeerId.')
generate_config_parser.add_argument('--node-name', dest='node_name', type=str, required=True,
                    help='Substrate node name.')
generate_config_parser.add_argument('--node-dir', dest='node_dir', type=str, default=None, nargs='?',
                    help='Substrate node name. (Default none)')
generate_config_parser.add_argument('--qkd-url', dest='qkd_url', type=url_type, required=True,
                    help='Qkd url.')
generate_config_parser.add_argument('--qkd-name', dest='qkd_name', type=str, required=True,
                    help='Qkd name.')
generate_config_parser.add_argument('--public-ip', dest='public_ip', type=ip_type, required=True,
                    help='Public ip.')
generate_config_parser.add_argument('--local-server-port', dest='local_server_port', default=5001, type=int, nargs='?',
                    help='Local server port. (Default 5001)')

generate_config_parser.add_argument('--external-server-port', dest='external_server_port', default=5002,
                    type=int, nargs='?', help='External server port (Default 5002)')

generate_config_parser.add_argument('--node-http-rpc-port', dest='node_http_rpc_port', default=9933, type=int, nargs='?',
                    help='Node http rpc port. (Default 9933)')
generate_config_parser.add_argument('--key-rotation-time', dest='key_rotation_time', default=50, type=int, nargs='?',
                    help='Key rotation time. (Default 50)')

generate_config_parser.add_argument('--recovery-check-interval', dest='recovery_check_interval', default=50, type=int,
                    nargs='?', help='Recovery check interval. (Default 50)')

generate_config_parser.add_argument('--qrng-api-key', dest='qrng_api_key', type=qrng_type, required=True,
                    help='Qrng api key.')

# get peers
get_peer_parser = subp.add_parser('get-peer', help='Get peer information.')
get_peer_parser.add_argument('--config-file', dest='config_file', default=50, type=str, required=True,
                    help='Path to config file.')
get_peer_parser.add_argument('--url', dest='boot_url', type=url_type, required=True, help='Boot url.')


run = subp.add_parser('run', help='Run node.')
run.add_argument('--config-file', '-c', dest='config_file', required=True, type=pathlib.Path,
                    nargs='?', help='Path to config file.')
run.add_argument(
    '--process', '-p', type=substrate_arguments,
    dest='startup_args', required=True,
    nargs='?', help='''The command required to start the node, containing all the necessary arguments for its operation.
    For example: --process "./target/release/qmc-node [arguments]" ''')


args = parser.parse_args()
