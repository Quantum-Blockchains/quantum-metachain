from psk.psk_file import create_psk_file, exists_psk_file, remove_psk_file
from os import path
from config import config


def test_create_and_remove_correct_psk():
    psk_data = "1234123412341234123412341234123412341234123412341234123412341234"
    create_psk_file(psk_data)

    assert exists_psk_file()

    with open(config.abs_psk_file_path()) as file:
        psk_file = file.read()

    assert psk_data == psk_file

    remove_psk_file()

    assert not path.exists(config.abs_psk_file_path())


def test_create_and_remove_psk_too_short():
    psk_data = "1234"
    create_psk_file(psk_data)

    assert exists_psk_file()

    with open(config.abs_psk_file_path()) as file:
        psk_file = file.read()

    assert psk_file == "0000000000000000000000000000000000000000000000000000000000001234"

    remove_psk_file()

    assert not path.exists(config.abs_psk_file_path())


def test_remove_with_no_file():
    try:
        remove_psk_file()
    except FileNotFoundError as e:
        print("Expected")

        return

    raise Exception("File was not removed but didn't receive error")
