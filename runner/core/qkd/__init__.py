from core.qkd.etsi014 import ETSI014Provider
from core.qkd.qnu_labs import QNULabsProvider


def get_qkd_provider(config):
    if config["provider"] == "qnulabs":
        return QNULabsProvider(config)
    else:
        return ETSI014Provider(config)
