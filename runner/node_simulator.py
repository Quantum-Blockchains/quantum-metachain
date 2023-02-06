import time
import config
from config import ConfigService, Config
from psk import exists_psk_file, remove_psk_file
import sys

path_config = sys.argv[2]
config.config_service = ConfigService(Config(path_config))

if exists_psk_file():
    remove_psk_file()

while True:
    time.sleep(50)
