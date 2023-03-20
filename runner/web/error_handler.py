import json

from flask import make_response, Flask
from requests.exceptions import RequestException
from common.logger import log
from common.exceptions import PeerMisconfiguredError, PSKNotFoundError


bad_request_response = {"error": "bad request"}
internal_server_error_response = {"error": "internal server error"}
not_found_response = {"error": "not found"}


def init_error_handlers(server: Flask):
    server.register_error_handler(400, handle_bad_request)
    server.register_error_handler(404, handle_not_found)
    server.register_error_handler(KeyError, handle_bad_request)
    server.register_error_handler(RequestException, handle_external_request_exception)
    server.register_error_handler(OSError, handle_os_exception)
    server.register_error_handler(Exception, handle_unexpected_exception)
    server.register_error_handler(PeerMisconfiguredError, handle_peer_misconfigured_error)
    server.register_error_handler(PSKNotFoundError, handle_psk_not_found_error)


def handle_bad_request(e):
    log.error(f"An error has occurred: {e}")
    return make_response(json.dumps(bad_request_response), 400)


def handle_not_found(_e):
    return make_response(json.dumps(not_found_response), 404)


def handle_external_request_exception(e):
    log.error(f"There was an error while sending request: {e}")
    return make_response(json.dumps(internal_server_error_response), 500)


def handle_unexpected_exception(e: Exception):
    log.error(f"An unexpected exception occurred: {e}")
    return make_response(json.dumps(internal_server_error_response), 500)


def handle_os_exception(e: OSError):
    log.error(f"There was an OS error: {e}")
    return make_response(json.dumps({internal_server_error_response}), 500)


def handle_peer_misconfigured_error(_e: PeerMisconfiguredError):
    return make_response(json.dumps({"message": "Peer misconfigured"}), 404)


def handle_psk_not_found_error(_e: PSKNotFoundError):
    return make_response(json.dumps({"message": "Pre shared key not found"}), 404)
