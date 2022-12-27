import sys
import os
import psk_file
import time
from config import Config

config_path = sys.argv[1]
config = Config(config_path)

if psk_file.exists(config):
    os.remove(config.abs_psk_file_path())

while True:
    time.sleep(50)
