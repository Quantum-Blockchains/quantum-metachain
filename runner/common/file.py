import os

from .config import config


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


psk_file_manager = FileManager(config.abs_psk_file_path())
node_key_file_manager = FileManager(config.abs_node_key_file_path())
psk_sig_file_manager = FileManager(config.abs_psk_sig_file_path())
