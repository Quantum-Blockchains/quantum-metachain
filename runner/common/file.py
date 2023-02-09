import os

from .config import config_service


class FileManager:

    def __init__(self, file_path):
        self.file_path = file_path

    def exists(self) -> bool:
        return os.path.exists(self.file_path)

    def create(self, data: str):
        with open(self.file_path, 'w') as file:
            file.write(data)

    def read(self) -> str:
        with open(self.file_path, 'r') as file:
            return file.read()

    def remove(self):
        if self.exists():
            os.remove(self.file_path)
        else:
            raise FileNotFoundError


psk_file_manager = None
node_key_file_manager = None
psk_sig_file_manager = None
