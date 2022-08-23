from django.urls import path
from rest_framework import routers

from app.views import root, projects, models, snapshots, deployments, console, notebooks
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

router = routers.DefaultRouter()
router.register("projects", projects.ProjectViewSet)
router.register("snapshots", snapshots.SnapshotViewSet)
router.register("models", models.ModelViewSet)
router.register("deployments", deployments.DeploymentViewSet)
router.register("tables", projects.TableView, basename="tables")
router.register("requests", root.RequestViewSet)

html_router = routers.DefaultRouter()
html_router.register("snapshots/analysis", snapshots.SnapshotAnalysisView, basename="snapshots/analysis")

urlpatterns = [
    path("", root.index, name="index"),
    path("deployments/", deployments.index, name="deployments"),
    path("deployments/<int:id>", deployments.deployment, name="deployment"),
    path("models/", models.ModelListView.as_view(), name="models"),
    path("models/<int:pk>", models.ModelView.as_view(), name="model"),
    path("projects/", projects.index, name="projects"),
    path("projects/new", projects.NewProjectView.as_view(), name="projects-new"),
    path("projects/<int:pk>", projects.ProjectView.as_view(), name="project"),
    path("snapshots/", snapshots.index, name="snapshots"),
    path("snapshots/<int:id>", snapshots.snapshot, name="snapshot"),
    path("console/", console.ConsoleView.as_view(), name="console"),
    path("console/run/", console.run_sql, name="console/run-sql"),
    path("set-auth-cookie/", root.set_auth_cookie, name="set-auth-cookie"),

    path("notebooks/", notebooks.index, name="notebooks"),
    path("notebooks/<int:pk>/", notebook, name="notebooks/notebook"),
    path("notebooks/create/", create_notebook, name="notebooks/create"),
    path("notebooks/<int:pk>/rename/", rename_notebook, name="notebooks/rename"),
    path("notebooks/<int:pk>/cell/add/", add_notebook_cell, name="notebooks/cell/add"),
    path("notebooks/<int:notebook_pk>/cell/<int:cell_pk>/", notebook_cell, name="notebooks/cell/get"),
    path("notebooks/<int:notebook_pk>/cell/<int:cell_pk>/edit/", edit_notebook_cell, name="notebooks/cell/edit"),
    path("notebooks/<int:notebook_pk>/cell/<int:cell_pk>/remove/", remove_notebook_cell, name="notebooks/cell/remove"),
    path(
        "notebooks/<int:notebook_pk>/cell/<int:cell_pk>/remove/undo/",
        undo_remove_notebook_cell,
        name="notebooks/cell/remove/undo",
    ),
    path("notebooks/<int:pk>/reset/", reset_notebook, name="notebooks/reset"),
    path("notebooks/<int:notebook_pk>/cell/<int:cell_pk>/play/", play_notebook_cell, name="notebooks/cell/play"),
]
