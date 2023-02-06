import argparse
import shlex
import pathlib


def substrate_arguments(string):
    substrate_parser = argparse.ArgumentParser()
    return substrate_parser.parse_known_args(shlex.split(string))[1]


parser = argparse.ArgumentParser(
    prog='runner/app.py',
    description='Quantum Blockchains runner.'
)

parser.add_argument('--config-file', '-c', dest='config_file', required=True, type=pathlib.Path,
                    nargs='?', help='Path to config file.')

parser.add_argument('--process', '-p', type=substrate_arguments, dest='startup_args', required=True,
                    nargs='?', help='Command for start node.')

args = parser.parse_args()
