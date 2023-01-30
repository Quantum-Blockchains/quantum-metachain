from psk_file import create, exists, remove
from os import path
from config import config


def test_create_and_remove_correct_psk():
    psk_data = "1234123412341234123412341234123412341234123412341234123412341234"
    create(psk_data)

    assert exists()

    with open(config.abs_psk_file_path()) as file:
        psk_file = file.read()

    assert psk_data == psk_file

    remove()

    assert not path.exists(config.abs_psk_file_path())


def test_create_and_remove_psk_too_short():
    psk_data = "1234"
    create(psk_data)

    assert exists()

    with open(config.abs_psk_file_path()) as file:
        psk_file = file.read()

    assert psk_file == "0000000000000000000000000000000000000000000000000000000000001234"

    remove()

    assert not path.exists(config.abs_psk_file_path())


def test_remove_with_no_file():
    try:
        remove()
    except FileNotFoundError as e:
        print("Expected")

        return

    raise
