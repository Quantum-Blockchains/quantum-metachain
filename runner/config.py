import json
from os import path


class Config():

    def __init__(self, config_path):
        self.PROJECT_DIR = path.abspath(path.dirname(__file__))
        self.ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
        self.ENV_PATH = path.join(self.ROOT_DIR, ".env")
        with open(f"{self.ROOT_DIR}/{config_path}", "r") as f:
            self.config = json.load(f)

    def abs_psk_file_path(self):
        return f"{self.ROOT_DIR}/{self.config['psk_file_path']}"
