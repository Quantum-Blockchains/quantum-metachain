from os import path

from config import abs_psk_file_path


def exists():
    return path.exists(abs_psk_file_path())


def create(psk):
    # Trim "0x" from psk
    if psk[:2] == "0x":
        psk = psk[2:]

    with open(abs_psk_file_path(), 'w') as file:
        file.write(psk)
