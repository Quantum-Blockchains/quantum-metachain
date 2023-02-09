import time
from common.file import psk_file_manager

if psk_file_manager.exists():
    psk_file_manager.remove()

while True:
    time.sleep(50)
