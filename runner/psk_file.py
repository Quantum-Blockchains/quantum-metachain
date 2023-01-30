import os
from os import path

from config import config


def exists():
    return path.exists(config.abs_psk_file_path())


def create(psk):
    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)


def remove():
    if exists():
        os.remove(config.abs_psk_file_path())
