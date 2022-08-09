from django.urls import path

from notebooks.views import (
    notebook,
    add_notebook_line,
    remove_notebook_line,
    notebook_line,
    edit_notebook_line,
    undo_remove_notebook_line,
)

urlpatterns = [
    path("notebook/<int:pk>/", notebook, name="notebooks/notebook"),
    path("notebook/<int:pk>/line/add/", add_notebook_line, name="notebooks/line/add"),
    path("notebook/<int:notebook_pk>/line/<int:line_pk>/", notebook_line, name="notebooks/line/get"),
    path("notebook/<int:notebook_pk>/line/<int:line_pk>/edit/", edit_notebook_line, name="notebooks/line/edit"),
    path("notebook/<int:notebook_pk>/line/<int:line_pk>/remove/", remove_notebook_line, name="notebooks/line/remove"),
    path(
        "notebook/<int:notebook_pk>/line/<int:line_pk>/remove/undo/",
        undo_remove_notebook_line,
        name="notebooks/line/remove/undo",
    ),
]
