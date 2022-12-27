import json
from os import path

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

with open(f"{ROOT_DIR}/config.json", "r") as f:
    config = json.load(f)


def abs_psk_file_path():
    return f"{ROOT_DIR}/{config['psk_file_path']}"

class Config():
    def __init__(self, config_path):
        self.PROJECT_DIR = path.abspath(path.dirname(__file__))
        self.ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
        self.ENV_PATH = path.join(ROOT_DIR, ".env")
        with open(f"{ROOT_DIR}/{config_path}", "r") as f:
            self.config = json.load(f)

    def abs_psk_file_path(self):
        return f"{self.ROOT_DIR}/{self.config['psk_file_path']}"

