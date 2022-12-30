from os import path
from config import config


def exists():
    return path.exists(config.abs_psk_file_path())


def create(psk):
    # Trim "0x" from psk
    if psk[:2] == "0x":
        psk = psk[2:]

    with open(config.abs_psk_file_path(), 'w') as file:
        file.write(psk)
