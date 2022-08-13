from django.shortcuts import render, reverse

from django.http import HttpResponse, HttpResponseRedirect

from app import models


def index(request):
    """Root of the dashboard."""
    return HttpResponseRedirect(reverse("notebooks"))


def set_auth_cookie(request):
    """Persist the user auth token in the cookie."""
    token = request.GET.get("dashboard_auth")

    if not token:
        return HttpResponse(status=400)

    response = HttpResponseRedirect(reverse("index"))
    response.set_cookie("dashboard_auth", token, secure=True, httponly=True)

    return response
