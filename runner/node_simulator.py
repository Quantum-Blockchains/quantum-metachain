import time
import sys
from common.file import psk_file_manager
from common import config

path_config = sys.argv[2]
config.config_service = ConfigService(Config(path_config))

if psk_file_manager.exists():
    psk_file_manager.remove()

while True:
    time.sleep(50)
