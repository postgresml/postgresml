from django.shortcuts import render

from django.http import HttpResponse

from .. import models


def index(request):
    projects = models.Project.objects.all()
    return render(request, "projects/index.html", {"title": "Projects", "projects": projects})
