from django.urls import path
from rest_framework import routers

from app.views import root, projects, models, snapshots, deployments

router = routers.DefaultRouter()
router.register("projects", projects.ProjectViewSet)
router.register("snapshots", snapshots.SnapshotViewSet)
router.register("models", models.ModelViewSet)
router.register("deployments", deployments.DeploymentViewSet)
router.register("tables", projects.TableView, basename="tables")

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
]
