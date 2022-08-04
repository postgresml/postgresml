from django.shortcuts import render, reverse

from django.http import HttpResponse, HttpResponseRedirect

from app import models


def index(request):
    projects = models.Project.objects.all()
    return render(request, "projects/index.html", {"title": "Projects", "projects": projects})


def set_auth_cookie(request):
    """Persist the user auth token in the cookie."""
    token = request.GET.get("dashboard_auth")

    if not token:
        return HttpResponse(status=400)

    response = HttpResponseRedirect(reverse("index"))
    response.set_cookie("dashboard_auth", token, secure=True, httponly=True)

    return response
