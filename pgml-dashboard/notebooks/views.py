from django.shortcuts import render, get_object_or_404
from django.urls import reverse_lazy, reverse
from django.http import HttpResponse, HttpResponseRedirect
from django import forms
from django.db import transaction

from notebooks.models import *


def notebook(request, pk):
    """Render a notebook."""
    notebook = get_object_or_404(Notebook, pk=pk)

    return render(
        request,
        "notebooks/notebook.html",
        {
            "lines": notebook.notebookline_set.all().order_by("pk"),
            "notebook": notebook,
        },
    )


class NotebookLineForm(forms.Form):
    contents = forms.CharField()


@transaction.atomic
def add_notebook_line(request, pk):
    """Add a new notebook line."""
    notebook = Notebook.objects.select_for_update().get(pk=pk)
    line_form = NotebookLineForm(request.POST)
    last_line = NotebookLine.objects.filter(notebook=notebook).order_by("line_number").last()

    if line_form.is_valid():
        contents = line_form.cleaned_data["contents"]

        if contents.startswith(r"%%sql"):
            line_type = NotebookLine.SQL
            contents = contents.replace(r"%%sql", "")
        else:
            line_type = NotebookLine.MARKDOWN

        line = NotebookLine.objects.create(
            notebook=notebook,
            contents=contents,
            line_number=(last_line.line_number + 1 if last_line else 1),
            line_type=line_type,
        )

    return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook.pk}))


@transaction.atomic
def edit_notebook_line(request, pk):
    line = get_object_or_404(NotebookLine, pk=pk)
    line_form = NotebookLineForm(request.POST)

    if line_form.is_valid():
        line = NotebookLine.objects.create(
            contents=line_form.cleaned_data["contents"],
            version=line.version + 1,
            line_number=line.line_number,
        )
        return HttpResponse(request, line.html())
    else:
        return HttpResponse(request, status=400)


@transaction.atomic
def remove_notebook_line(request, notebook_pk, line_pk):
    line = get_object_or_404(NotebookLine, pk=line_pk, notebook__pk=notebook_pk)
    line.delete()

    return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook_pk}))
