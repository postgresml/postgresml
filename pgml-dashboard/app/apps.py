from django.apps import AppConfig
from django.core.serializers import register_serializer


class AppConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "app"

    def ready(self):
        register_serializer("yml", "django.core.serializers.pyyaml")
