import jwt
from django.conf import settings
from django.http import HttpResponse
from django.shortcuts import reverse
from django.db import connection

PERMISSION_DENIED = """
<!DOCTYPE html>
<html lang="en_US">
    <head>
    <meta charset="utf-8">
    <title>Permission denied</title>
    <body>
        <h1>Permission denied</h1>
    </body>
</html>
"""


class JwtAuthentcationMiddleware:
    """Validate an incoming request is JWT authenticated."""

    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        # Don't do this unless the auth is enabled
        if not getattr(settings, "JWT_AUTH_ENABLED"):
            return self.get_response(request)

        if request.path.startswith(reverse("set-auth-cookie")):
            return self.get_response(request)

        try:
            # First check the cookie
            token = request.COOKIES.get("dashboard_auth")

            # Try
            if not token:
                auth_header = request.META.get("HTTP_AUTHORIZATION")

                if not auth_header:
                    return HttpResponse(PERMISSION_DENIED, status=403)

                # Extract the token assuming "Bearer <token>"
                token = auth_header.split(" ")[-1]

            print(token)
            token = jwt.decode(token, settings.SECRET_KEY, algorithms=["HS256"])
            request.dashboard_uuid = token["dashboard"]
            if "shard" in token:
                request.shard = int(token["shard"])
        except Exception as e:
            print(e)
            return HttpResponse(PERMISSION_DENIED, status=403)

        return self.get_response(request)


class MultiTenantMiddleware:
    """Handle dashboard using multiple PostgresML instances.

    PostgresML instances must be proxied by PgCat."""

    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        if hasattr(request, "shard"):
            with connection.cursor() as cursor:
                shard = request.shard
                # https://github.com/levkk/pgcat
                cursor.execute(f"SET SHARD TO {shard}")
        return self.get_response(request)
