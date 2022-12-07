from os import path

from config import config


def exists():
    return path.exists(config["psk_file_path"])


def create(psk):
    with open(config["psk_file_path"], 'w') as file:
        file.write(psk)
