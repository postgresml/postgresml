from django.shortcuts import render, get_object_or_404

from .. import models

def default_context(context):
    return {"topic": "Projects", **context}

def index(request):
    projects = models.Project.objects.all()
    context = default_context({'title': 'Projects', 'projects': projects})
    return render(request, 'projects/index.html', context)

def project(request, id):
    if request.method == "GET":
        return get(request, id)

def get(request, id):
    project = get_object_or_404(models.Project, id=id)
    context = default_context({'title': project.name, 'project': project})
    return render(request, 'projects/project.html', context)

