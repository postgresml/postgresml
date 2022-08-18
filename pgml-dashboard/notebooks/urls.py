from django.urls import path

from notebooks.views import (
    notebook,
    add_notebook_cell,
    remove_notebook_cell,
    notebook_cell,
    edit_notebook_cell,
    undo_remove_notebook_cell,
    create_notebook,
    rename_notebook,
    reset_notebook,
    play_notebook_cell,
)

urlpatterns = [
    path("notebook/<int:pk>/", notebook, name="notebooks/notebook"),
    path("create/", create_notebook, name="notebooks/create"),
    path("notebook/<int:pk>/rename/", rename_notebook, name="notebooks/rename"),
    path("notebook/<int:pk>/cell/add/", add_notebook_cell, name="notebooks/cell/add"),
    path("notebook/<int:notebook_pk>/cell/<int:cell_pk>/", notebook_cell, name="notebooks/cell/get"),
    path("notebook/<int:notebook_pk>/cell/<int:cell_pk>/edit/", edit_notebook_cell, name="notebooks/cell/edit"),
    path("notebook/<int:notebook_pk>/cell/<int:cell_pk>/remove/", remove_notebook_cell, name="notebooks/cell/remove"),
    path(
        "notebook/<int:notebook_pk>/cell/<int:cell_pk>/remove/undo/",
        undo_remove_notebook_cell,
        name="notebooks/cell/remove/undo",
    ),
    path("notebook/<int:pk>/reset/", reset_notebook, name="notebooks/reset"),
    path("notebook/<int:notebook_pk>/cell/<int:cell_pk>/play/", play_notebook_cell, name="notebooks/cell/play"),
]
