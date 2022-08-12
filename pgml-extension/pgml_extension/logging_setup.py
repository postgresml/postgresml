import logging

try:
    import plpy
except:
    plpy = None


class ListenFilter(logging.Filter):
    def filter(self, record):
        """Determine which log records to output.
        Returns True for unfiltered messages. False for filtered.
        """
        return True


class RequestsHandler(logging.Handler):
    """Intercepts messages to python logging and forwards them on to plpy."""

    def emit(self, record):
        """Send the log records to the appropriate destination."""
        if (record.levelno is None or record.levelno <= 10) and False:
            plpy.debug(record.getMessage())
        elif record.levelno <= 20 and False:
            plpy.info(record.getMessage())
        elif record.levelno <= 30:
            plpy.warning(record.getMessage())
        elif record.levelno <= 40:
            plpy.error(record.getMessage())
        else:
            plpy.critical(record.getMessage())


if plpy:
    # Let Postgres implement the log level
    logger = logging.getLogger()
    logger.setLevel(logging.DEBUG)
    handler = RequestsHandler()
    logger.addHandler(handler)
    filter_ = ListenFilter()
    logger.addFilter(filter_)
