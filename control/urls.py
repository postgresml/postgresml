from django.urls import path

from . import views
from .views import root, projects, models, snapshots, deployments


urlpatterns = [
    path('', root.index, name='index'),
    path('deployments/', deployments.index, name='deployments'),
    path('deployments/<int:id>', deployments.deployment, name='deployment'),
    path('models/', models.index, name='models'),
    path('models/<int:id>', models.model, name='model'),
    path('projects/', projects.index, name='projects'),
    path('projects/<int:id>', projects.project, name='project'),
    path('snapshots/', snapshots.index, name='snapshots'),
    path('snapshots/<int:id>', snapshots.snapshot, name='snapshot'),
]
