import time
import psk_file

if psk_file.exists():
    psk_file.remove()

while True:
    time.sleep(50)
