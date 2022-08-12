from contextlib import contextmanager
from datetime import datetime
import logging
import re

_nested_timers = 0
_previous_timer_level = 0
_ascii_pipes = "  "
_timer_logger = logging.getLogger("postgresml.timer")


@contextmanager
def timer(message="elapsed time:", level=logging.INFO, logger=None):
    global _nested_timers, _previous_timer_level, _ascii_pipes, _timer_logger

    if logger is None:
        logger = _timer_logger

    if level < logger.level:
        yield
        return
    _nested_timers += 1
    start = datetime.now()
    try:
        yield
    finally:
        time = datetime.now() - start

        _nested_timers -= 1
        if _nested_timers == 0:
            _ascii_pipes = ""
        else:
            delta = _nested_timers - _previous_timer_level
            length = _nested_timers * 2
            if delta < 0:
                _ascii_pipes = _ascii_pipes[0:length]
                join = "┌" if _ascii_pipes[-2] == " " else "├"
                _ascii_pipes = _ascii_pipes[0:-2] + join + "─"
            else:
                _ascii_pipes = re.sub(r"[├┌]", "│", _ascii_pipes).replace("─", " ")
                if delta == 0:
                    _ascii_pipes = _ascii_pipes[:-2] + "├─"
                else:
                    gap = length - len(_ascii_pipes) - 2
                    _ascii_pipes = _ascii_pipes + " " * gap + "┌─"

        _previous_timer_level = _nested_timers
        logger.log(level, (_ascii_pipes + "[" + str(time) + "] " + message))
