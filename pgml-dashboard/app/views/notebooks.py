from django.shortcuts import render
from notebooks.models import Notebook


def index(request):
    return render(
        request,
        "notebooks.html",
        {
            "notebooks": Notebook.objects.all(),
            "topic": "notebooks",
        },
    )
