import os
from os import path

from config import config
from runner.utils import trim_0x_prefix


def exists():
    return path.exists(config.abs_psk_file_path())


def create(psk):
    psk = trim_0x_prefix(psk)
    while len(psk) < 64:
        psk = "0" + psk

    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)


def remove():
    if exists():
        os.remove(config.abs_psk_file_path())
