from os import path, getenv

import dotenv

PROJECT_DIR = path.abspath(path.dirname(__file__))
ROOT_DIR = path.abspath(path.dirname(__file__) + "/..")
ENV_PATH = path.join(ROOT_DIR, ".env")

defaults = {
    "LOCAL_SERVER_PORT": 5001,
    "EXTERNAL_SERVER_PORT": 5002,
    "KEY_ROTATION_TIME": 600,
    "PSK_FILE_PATH": f"{ROOT_DIR}/psk",
    "QRNG_API_KEY": ""
}


class Settings:

    def __init__(self):
        dotenv.load_dotenv(ENV_PATH)

    def __getattribute__(self, name):
        default = defaults.get(name)
        if getenv(name, default) != dotenv.get_key(ENV_PATH, name):
            dotenv.load_dotenv(ENV_PATH, override=True)
        return getenv(name, default)

    def __setattr__(self, item, value):
        dotenv.set_key(ENV_PATH, item, value)
        dotenv.load_dotenv(ENV_PATH, override=True)


settings = Settings()
