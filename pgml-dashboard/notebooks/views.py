from django.shortcuts import render, get_object_or_404
from django.urls import reverse_lazy, reverse
from django.http import HttpResponse, HttpResponseRedirect
from django import forms
from django.db import transaction
from django.utils import timezone

from notebooks.models import *


def notebook(request, pk):
    """Render a notebook."""
    notebook = get_object_or_404(Notebook, pk=pk)

    return render(
        request,
        "notebooks/notebook.html",
        {
            "lines": notebook.notebookline_set.all().filter(deleted_at__isnull=True).order_by("line_number"),
            "notebook": notebook,
        },
    )


class NotebookForm(forms.Form):
    name = forms.CharField()


def create_notebook(request):
    notebook_form = NotebookForm(request.POST)

    if notebook_form.is_valid():
        notebook = Notebook.objects.create(
            name=notebook_form.cleaned_data["name"],
        )

        return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook.pk}))
    else:
        return HttpResponse(status=400)


def rename_notebook(request, pk):
    notebook_form = NotebookForm(request.POST)

    if notebook_form.is_valid():
        Notebook.objects.filter(pk=pk).update(name=notebook_form.cleaned_data["name"])
        return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": pk}))
    else:
        return HttpResponse(status=400)


def notebook_line(request, notebook_pk, line_pk):
    """Render a single notebook line."""
    notebook = get_object_or_404(Notebook, pk=notebook_pk)
    line = get_object_or_404(NotebookLine, pk=line_pk)

    return render(
        request,
        "notebooks/line.html",
        {
            "line": line,
            "notebook": line.notebook,
        },
    )


class NotebookLineForm(forms.Form):
    contents = forms.CharField(required=False)


@transaction.atomic
def add_notebook_line(request, pk):
    """Add a new notebook line."""
    notebook = Notebook.objects.select_for_update().get(pk=pk)
    line_form = NotebookLineForm(request.POST)
    last_line = NotebookLine.objects.filter(notebook=notebook, deleted_at__isnull=True).order_by("line_number").last()

    if line_form.is_valid():
        contents = line_form.cleaned_data["contents"].strip()

        if contents.startswith(r"%%sql"):
            line_type = NotebookLine.SQL
        else:
            line_type = NotebookLine.MARKDOWN

        line = NotebookLine.objects.create(
            notebook=notebook,
            contents=contents,
            line_number=(last_line.line_number + 1 if last_line else 1),
            line_type=line_type,
        )

        return HttpResponseRedirect(
            reverse_lazy("notebooks/line/get", kwargs={"notebook_pk": notebook.pk, "line_pk": line.pk})
        )
    else:
        print(line_form.errors)
        return HttpResponse(line_form.errors, status=400)


def edit_notebook_line(request, notebook_pk, line_pk):
    notebook = get_object_or_404(Notebook, pk=notebook_pk)
    old_line = get_object_or_404(NotebookLine, pk=line_pk)
    line_form = NotebookLineForm(request.POST)

    if line_form.is_valid():
        contents = line_form.cleaned_data["contents"].strip()

        if contents.startswith(r"%%sql"):
            line_type = NotebookLine.SQL
        else:
            line_type = NotebookLine.MARKDOWN

        with transaction.atomic():
            new_line = NotebookLine.objects.create(
                notebook=notebook,
                contents=contents,
                version=old_line.version + 1,
                line_number=old_line.line_number,
                line_type=line_type,
            )
            old_line.delete()
        return render(
            request,
            "notebooks/line.html",
            {
                "line": new_line,
                "notebook": notebook,
            },
        )
    else:
        return HttpResponse(request, status=400)


@transaction.atomic
def remove_notebook_line(request, notebook_pk, line_pk):
    line = get_object_or_404(NotebookLine, pk=line_pk, notebook__pk=notebook_pk)
    line.deleted_at = timezone.now()
    line.save()

    return render(
        request,
        "notebooks/undo.html",
        {
            "line": line,
            "notebook": line.notebook,
        },
    )


def undo_remove_notebook_line(request, notebook_pk, line_pk):
    line = get_object_or_404(NotebookLine, pk=line_pk, notebook__pk=notebook_pk)
    line.deleted_at = None
    line.save()

    return render(
        request,
        "notebooks/line.html",
        {
            "line": line,
            "notebook": line.notebook,
        },
    )
