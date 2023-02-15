import common.config
import common.file


def pytest_sessionstart(session):
    common.config.init_config()
    common.file.initialise_file_managers()
