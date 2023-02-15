import time
import sys
from common.file import FileManager
from common.config import ConfigService, Config
import common.config
import common.file

path_config = sys.argv[2]
common.config.init_config(path_config)
common.file.psk_file_manager = FileManager(common.config.config_service.current_config.abs_psk_file_path())

if common.file.psk_file_manager.exists():
    common.file.psk_file_manager.remove()

while True:
    time.sleep(50)
