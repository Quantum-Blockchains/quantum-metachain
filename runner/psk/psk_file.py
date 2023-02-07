import os
from os import path
from config import config
from utils import trim_0x_prefix


def exists_psk_file():
    return path.exists(config.abs_psk_file_path())


def create_psk_file(psk):
    psk = trim_0x_prefix(psk)
    while len(psk) < 64:
        psk = "0" + psk

    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)


def remove_psk_file():
    if exists_psk_file():
        os.remove(config.abs_psk_file_path())
