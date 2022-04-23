from django.shortcuts import render

from .. import models

def index(request):
    projects = models.Project.objects.all()
    return render(request, 'projects/project_listing.html', {'title': 'Projects', 'projects': projects})
