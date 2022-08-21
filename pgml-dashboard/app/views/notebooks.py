from django.shortcuts import render
from notebooks.models import Notebook


def index(request):
    return render(
        request,
        "notebooks.html",
        {
            "notebooks": Notebook.objects.order_by('id').all(),
            "topic": "notebooks",
        },
    )
