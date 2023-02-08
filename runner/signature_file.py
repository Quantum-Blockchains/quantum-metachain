from config import config


def create(signature: str):
    with open(config.abs_psk_sig_file_path(), 'w') as file:
        file.write(signature)
