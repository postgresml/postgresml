from django.conf import settings


def url_prefix(request):
    url_prefix = settings.URL_PREFIX
    if url_prefix.endswith("/"):
        url_prefix = url_prefix[:-1]
    return {
        "url_prefix": url_prefix,
    }
