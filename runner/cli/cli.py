import cli.params
from cli.generate_node_key import generate_node_key
from cli.generate_config_node import generate_config_node
from cli.get_peer import get_peer
from cli.run import run
from cli.generate_config_node_wizard import generate_config_node_wizard


def start():
    match cli.params.args.command:
        case 'generate-node-key':
            generate_node_key(cli.params.args)
        case 'generate-config-node':
            generate_config_node(cli.params.args)
        case 'generate-config-node-wizard':
            generate_config_node_wizard()
        case 'get-peer':
            get_peer(cli.params.args)
        case 'run':
            run(cli.params.args)
        case _:
            pass