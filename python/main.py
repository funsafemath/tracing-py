import inspect
from functools import wraps
from typing import Callable

import tracing


def instrument(function: Callable):
    signature = inspect.signature(function)

    @wraps(function)
    def traced(*args, **kwargs):
        print(args, kwargs)
        for el in signature.parameters:
            print(el)
        return function(*args, **kwargs)

    return traced


tracing.init()

import time
import logging

t = time.time()
for i in range(100000):
    # logging.error("error")
    # logging.error("error")
    # logging.error("error")
    # logging.debug("error")
    # logging.debug("error")
    tracing.error("error")
    tracing.warn("warn")
    tracing.info("info")
    tracing.debug("debug")
    tracing.trace("trace")
print(time.time() - t)
