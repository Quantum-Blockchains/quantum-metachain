import os
import psk_file
import time
from config import config


if psk_file.exists():
    os.remove(config.abs_psk_file_path())

while True:
    time.sleep(50)
