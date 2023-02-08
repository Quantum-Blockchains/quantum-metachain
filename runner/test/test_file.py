import tempfile

import pytest

from common.file import FileManager


data = "test_data"


@pytest.fixture()
def before_each():
    pytest.file_manager = FileManager(f"{tempfile.gettempdir()}/test.txt")


def test_file_exists(before_each):
    pytest.file_manager.create(data)
    assert pytest.file_manager.exists()


def test_can_read_file_data(before_each):
    pytest.file_manager.create(data)
    assert pytest.file_manager.read() == data


def test_can_remove_the_file(before_each):
    pytest.file_manager.create(data)
    pytest.file_manager.remove()
    assert not pytest.file_manager.exists()


def test_remove_when_file_does_not_exist_throws_an_error(before_each):
    try:
        pytest.file_manager.remove()
    except FileNotFoundError:
        print("Expected")

        return

    raise Exception("File was not removed but didn't receive error")
