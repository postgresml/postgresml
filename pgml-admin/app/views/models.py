from django.shortcuts import render, get_object_or_404

from ..models import Model


def default_context(context):
    return {"topic": "Models", **context}


def index(request):
    models = Model.objects.all()
    context = default_context({"title": "Models", "models": models})
    return render(request, "models/index.html", context)


def model(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    model = get_object_or_404(Model, id=id)
    context = default_context({"title": model.project.name + " - " + model.algorithm_name, "model": model})
    return render(request, "models/model.html", context)
