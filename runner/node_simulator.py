import time
from psk import exists_psk_file, remove_psk_file

if exists_psk_file():
    remove_psk_file()

while True:
    time.sleep(50)
