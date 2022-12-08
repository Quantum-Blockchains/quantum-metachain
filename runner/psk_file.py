from os import path

from config import abs_psk_file_path


def exists():
    return path.exists(abs_psk_file_path())


def create(psk):
    with open(abs_psk_file_path(), 'w') as file:
        file.write(psk)
