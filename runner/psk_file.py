import os
from os import path

from config import settings


def exists():
    return path.exists(settings.PSK_FILE_PATH)


def remove():
    os.remove(settings.PSK_FILE_PATH)


def create(psk):
    with open(settings.PSK_FILE_PATH, 'w') as file:
        file.write(psk)
