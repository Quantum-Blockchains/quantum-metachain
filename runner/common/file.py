import os
import common.config

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


global psk_file_manager, node_key_file_manager, psk_sig_file_manager, config_file_manager, node_logs_file_manager,\
    runner_logs_file_manager, local_qkd_target_file_manager


# def initialise_file_managers(config_path):
#     global psk_file_manager, node_key_file_manager, psk_sig_file_manager, config_file_manager
#     psk_file_manager = FileManager(common.config.config_service.config.psk_file_path)
#     node_key_file_manager = FileManager(common.config.config_service.config.node_key_file_path)
#     psk_sig_file_manager = FileManager(common.config.config_service.config.psk_sig_file_path)
#     config_file_manager = FileManager(common.config.to_absolute(config_path))
def initialise_file_managers(node_dir):
    global psk_file_manager, node_key_file_manager, psk_sig_file_manager, config_file_manager, node_logs_file_manager,\
        runner_logs_file_manager, local_qkd_target_file_manager
    psk_file_manager = FileManager(os.path.join(node_dir, "psk"))
    node_key_file_manager = FileManager(os.path.join(node_dir, "node_key"))
    psk_sig_file_manager = FileManager(os.path.join(node_dir, "psk_sig"))
    config_file_manager = FileManager(os.path.join(node_dir, "config"))
    node_logs_file_manager = FileManager(os.path.join(node_dir, "logs/node.log"))
    runner_logs_file_manager = FileManager(os.path.join(node_dir, "logs/runner.log"))
    local_qkd_target_file_manager = FileManager(os.path.join(node_dir, common.config.config_service.config.local_qkd_target))
