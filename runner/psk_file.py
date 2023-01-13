from os import path
from config import config
import os


def exists():
    return path.exists(config.abs_psk_file_path())


def create(psk):
    # Trim "0x" from psk
    if psk[:2] == "0x":
        psk = psk[2:]
    while len(psk) < 64:
        psk = "0" + psk

    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)


def remove():
    if exists():
        os.remove(config.abs_psk_file_path())
