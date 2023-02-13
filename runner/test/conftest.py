import common.config
import common.file


def pytest_sessionstart(session):
    common.config.config_service = common.config.ConfigService(common.config.Config())
    common.file.initialise_file_managers()
