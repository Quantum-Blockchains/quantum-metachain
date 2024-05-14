import ipaddress
import argparse
from urllib.parse import urlparse
import base58
import shlex


def uint_type(arg):
    try:
        i = int(arg)
        if i < 0:
            raise argparse.ArgumentTypeError("The number must be equal to or greater than 0.")
    except Exception as err:
        raise argparse.ArgumentTypeError("The number must be equal to or greater than 0.")
    return i


def port_type(arg):
    try:
        i = int(arg)
        if (not i > 0) or (not i < 2 ** 16):
            raise argparse.ArgumentTypeError("Port numbers must be integers between 0 and 2**16")
    except Exception as err:
        raise argparse.ArgumentTypeError("Port numbers must be integers between 0 and 2**16")
    return i


def ip_type(arg):
    try:
        ipaddress.ip_address(arg)
        return arg
    except Exception as err:
        raise argparse.ArgumentTypeError('Invalid IP address')


def url_type(arg):
    url = urlparse(arg)
    if all((url.scheme, url.netloc)):
        return arg
    raise argparse.ArgumentTypeError('Invalid URL')


def qrng_type(arg):
    qrng_key = arg.split('-')
    if qrng_key.__len__() != 5:
        raise argparse.ArgumentTypeError('Invalid QRNG api key')
    elif (qrng_key[0].__len__() != 8 or qrng_key[1].__len__() != 4 or qrng_key[2].__len__() != 4 or
          qrng_key[3].__len__() != 4 or qrng_key[4].__len__() != 12):
        raise argparse.ArgumentTypeError('Invalid QRNG api key')
    return arg


def peer_type(arg):
    if arg.__len__() != 52:
        raise argparse.ArgumentTypeError('Invalid peer')
    try:
        decoded = base58.b58decode(arg)
        return arg
    except base58.base58.InvalidBase58Error:
        raise argparse.ArgumentTypeError('Invalid peer')


def substrate_arguments(string):
    substrate_parser = argparse.ArgumentParser()
    return substrate_parser.parse_known_args(shlex.split(string))[1]