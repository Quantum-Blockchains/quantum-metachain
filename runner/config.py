import json
from os import path

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

with open(f"{ROOT_DIR}/config.json", "r") as f:
    config = json.load(f)


def abs_psk_file_path():
    return f"{ROOT_DIR}/{config['psk_file_path']}"
