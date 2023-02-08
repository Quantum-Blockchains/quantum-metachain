import os
from os import path
from config import config


def exists_psk_file():
    return path.exists(config.abs_psk_file_path())


def create_psk_file(psk):
    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)


def remove_psk_file():
    if exists_psk_file():
        os.remove(config.abs_psk_file_path())
