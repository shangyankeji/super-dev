---
id: django-complete
title: Django 完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, complete, django, framework, rest, 中间件, 数据库迁移, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Django 完整指南

## 概述

Django 是 Python 生态中最成熟的全栈 Web 框架，遵循 MTV（Model-Template-View）模式，内置 ORM、认证系统、后台管理、表单处理、国际化等企业级功能。Django 秉承"batteries included"哲学，适用于从原型到大规模生产的全生命周期开发。

### 核心特性

- **ORM**: 强大的对象关系映射，支持多种数据库后端
- **Admin**: 自动生成的后台管理界面
- **认证系统**: 内置用户认证、权限、组管理
- **迁移系统**: 数据库 schema 版本控制
- **安全**: 内置 CSRF/XSS/SQL 注入/Clickjacking 防护
- **缓存框架**: 多级缓存策略，支持 Redis/Memcached
- **信号系统**: 松耦合的事件驱动扩展机制
- **中间件**: 请求/响应处理管线

### 为什么选择 Django?

- 成熟稳定，社区活跃（GitHub 70k+ stars）
- 文档质量业界公认一流
- Instagram/Pinterest/Disqus/Mozilla 等大规模验证
- 丰富的第三方生态（DRF/Celery/Channels/django-allauth 等）

---

## 项目结构最佳实践

### 标准项目布局

```
project_root/
├── manage.py
├── requirements/
│   ├── base.txt
│   ├── dev.txt
│   └── prod.txt
├── config/                      # 项目配置包（推荐重命名）
│   ├── __init__.py
│   ├── settings/
│   │   ├── __init__.py
│   │   ├── base.py              # 公共配置
│   │   ├── dev.py               # 开发环境
│   │   ├── staging.py           # 预发布环境
│   │   └── prod.py              # 生产环境
│   ├── urls.py
│   ├── wsgi.py
│   └── asgi.py
├── apps/
│   ├── users/
│   │   ├── __init__.py
│   │   ├── admin.py
│   │   ├── apps.py
│   │   ├── models.py
│   │   ├── managers.py          # 自定义 Manager
│   │   ├── serializers.py
│   │   ├── views.py
│   │   ├── urls.py
│   │   ├── signals.py
│   │   ├── tasks.py             # Celery 任务
│   │   ├── permissions.py
│   │   ├── tests/
│   │   │   ├── __init__.py
│   │   │   ├── test_models.py
│   │   │   ├── test_views.py
│   │   │   └── factories.py     # Factory Boy
│   │   └── migrations/
│   └── orders/
│       └── ...
├── templates/
├── static/
├── media/
├── locale/
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
└── docs/
```

### 设置拆分最佳实践

```python
# config/settings/base.py
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent.parent

SECRET_KEY = os.environ.get("DJANGO_SECRET_KEY")

INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    # 第三方
    "rest_framework",
    "django_filters",
    "corsheaders",
    # 本地
    "apps.users",
    "apps.orders",
]

AUTH_USER_MODEL = "users.User"

# config/settings/dev.py
from .base import *  # noqa: F401,F403

DEBUG = True
ALLOWED_HOSTS = ["*"]
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": "myproject_dev",
        "HOST": "localhost",
        "PORT": "5432",
    }
}

# config/settings/prod.py
from .base import *  # noqa: F401,F403

DEBUG = False
ALLOWED_HOSTS = os.environ.get("ALLOWED_HOSTS", "").split(",")
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": os.environ["DB_NAME"],
        "USER": os.environ["DB_USER"],
        "PASSWORD": os.environ["DB_PASSWORD"],
        "HOST": os.environ["DB_HOST"],
        "PORT": os.environ.get("DB_PORT", "5432"),
        "CONN_MAX_AGE": 600,
        "OPTIONS": {
            "connect_timeout": 10,
            "options": "-c statement_timeout=30000",
        },
    }
}
```

---

## ORM 高级用法

### 模型定义

```python
from django.db import models
from django.utils import timezone


class TimeStampedModel(models.Model):
    """可复用的时间戳抽象基类"""
    created_at = models.DateTimeField(auto_now_add=True, db_index=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        abstract = True


class Category(models.Model):
    name = models.CharField(max_length=100, unique=True)
    slug = models.SlugField(unique=True)
    parent = models.ForeignKey(
        "self", null=True, blank=True,
        on_delete=models.CASCADE,
        related_name="children",
    )

    class Meta:
        verbose_name_plural = "Categories"
        ordering = ["name"]
        indexes = [
            models.Index(fields=["slug"]),
        ]

    def __str__(self):
        return self.name


class Article(TimeStampedModel):
    class Status(models.TextChoices):
        DRAFT = "draft", "Draft"
        PUBLISHED = "published", "Published"
        ARCHIVED = "archived", "Archived"

    title = models.CharField(max_length=200)
    slug = models.SlugField(max_length=200, unique_for_date="publish_date")
    author = models.ForeignKey(
        "users.User",
        on_delete=models.CASCADE,
        related_name="articles",
    )
    category = models.ForeignKey(
        Category, on_delete=models.SET_NULL,
        null=True, related_name="articles",
    )
    body = models.TextField()
    status = models.CharField(
        max_length=10,
        choices=Status.choices,
        default=Status.DRAFT,
        db_index=True,
    )
    publish_date = models.DateTimeField(default=timezone.now)
    tags = models.ManyToManyField("Tag", blank=True, related_name="articles")
    view_count = models.PositiveIntegerField(default=0)

    class Meta:
        ordering = ["-publish_date"]
        indexes = [
            models.Index(fields=["-publish_date", "status"]),
            models.Index(fields=["author", "status"]),
        ]
        constraints = [
            models.CheckConstraint(
                check=models.Q(view_count__gte=0),
                name="view_count_non_negative",
            ),
        ]
```

### QuerySet 高级查询

```python
from django.db.models import Q, F, Count, Avg, Sum, Subquery, OuterRef, Exists

# F 表达式 —— 引用字段值进行数据库级操作
Article.objects.filter(updated_at__gt=F("created_at"))
Article.objects.update(view_count=F("view_count") + 1)

# Q 对象 —— 构建复杂查询条件
Article.objects.filter(
    Q(status="published") & (Q(title__icontains="django") | Q(body__icontains="django"))
)

# 排除条件
Article.objects.exclude(
    Q(status="draft") | Q(author__is_active=False)
)

# annotate —— 为每行附加聚合计算值
authors_with_stats = User.objects.annotate(
    article_count=Count("articles"),
    avg_views=Avg("articles__view_count"),
    total_views=Sum("articles__view_count"),
).filter(article_count__gt=0).order_by("-total_views")

# aggregate —— 全表聚合
from django.db.models import Max, Min
stats = Article.objects.filter(status="published").aggregate(
    total=Count("id"),
    avg_views=Avg("view_count"),
    max_views=Max("view_count"),
    min_views=Min("view_count"),
)

# Subquery —— 子查询
newest_article = Article.objects.filter(
    author=OuterRef("pk")
).order_by("-publish_date")

users_with_latest = User.objects.annotate(
    latest_article_title=Subquery(newest_article.values("title")[:1]),
    latest_article_date=Subquery(newest_article.values("publish_date")[:1]),
)

# Exists —— 高效存在性检查
active_authors = User.objects.filter(
    Exists(Article.objects.filter(author=OuterRef("pk"), status="published"))
)
```

### Prefetch 与 select_related

```python
from django.db.models import Prefetch

# select_related —— ForeignKey / OneToOne，单次 JOIN
articles = Article.objects.select_related("author", "category").all()

# prefetch_related —— ManyToMany / 反向 FK，独立查询后合并
articles = Article.objects.prefetch_related("tags").all()

# Prefetch 对象 —— 自定义预取 QuerySet
published_articles = Article.objects.filter(status="published")
users = User.objects.prefetch_related(
    Prefetch(
        "articles",
        queryset=published_articles.select_related("category"),
        to_attr="published_articles",  # 结果存到属性而非 Manager
    )
)

# 嵌套预取
categories = Category.objects.prefetch_related(
    Prefetch(
        "articles",
        queryset=Article.objects.select_related("author").prefetch_related("tags"),
    )
)
```

### 自定义 Manager 和 QuerySet

```python
class PublishedQuerySet(models.QuerySet):
    def published(self):
        return self.filter(status="published", publish_date__lte=timezone.now())

    def by_author(self, user):
        return self.filter(author=user)

    def popular(self, min_views=100):
        return self.filter(view_count__gte=min_views)

    def with_stats(self):
        return self.annotate(
            comment_count=Count("comments"),
            avg_rating=Avg("ratings__score"),
        )


class PublishedManager(models.Manager):
    def get_queryset(self):
        return PublishedQuerySet(self.model, using=self._db).published()


class Article(TimeStampedModel):
    # ...字段定义...
    objects = models.Manager()             # 默认 Manager
    published = PublishedManager()          # 自定义 Manager

    class Meta:
        default_manager_name = "objects"

# 链式调用
Article.published.by_author(user).popular().with_stats()
```

---

## 数据库迁移

### 迁移管理

```bash
# 生成迁移文件
python manage.py makemigrations

# 查看迁移 SQL（不执行）
python manage.py sqlmigrate myapp 0001

# 执行迁移
python manage.py migrate

# 查看迁移状态
python manage.py showmigrations

# 回滚到指定迁移
python manage.py migrate myapp 0003

# 生成空迁移（用于数据迁移）
python manage.py makemigrations myapp --empty -n populate_slugs
```

### 数据迁移

```python
from django.db import migrations

def populate_slugs(apps, schema_editor):
    from django.utils.text import slugify
    Article = apps.get_model("myapp", "Article")
    for article in Article.objects.filter(slug=""):
        article.slug = slugify(article.title)
        article.save(update_fields=["slug"])

def reverse_slugs(apps, schema_editor):
    pass  # 反向迁移通常不需要操作

class Migration(migrations.Migration):
    dependencies = [("myapp", "0005_article_slug")]

    operations = [
        migrations.RunPython(populate_slugs, reverse_slugs),
    ]
```

### 迁移最佳实践

- 每次 `makemigrations` 只修改一个 app
- 数据迁移和 schema 迁移分开写
- 大表添加列时使用 `db_default`（Django 5.0+）或分步迁移
- 避免在迁移中直接 import Model，使用 `apps.get_model()`
- 生产环境迁移前先在 staging 测试
- 使用 `--check` 在 CI 中验证迁移文件是否最新

---

## 认证与权限

### 自定义 User 模型

```python
from django.contrib.auth.models import AbstractUser, BaseUserManager


class UserManager(BaseUserManager):
    def create_user(self, email, password=None, **extra_fields):
        if not email:
            raise ValueError("Email is required")
        email = self.normalize_email(email)
        user = self.model(email=email, **extra_fields)
        user.set_password(password)
        user.save(using=self._db)
        return user

    def create_superuser(self, email, password=None, **extra_fields):
        extra_fields.setdefault("is_staff", True)
        extra_fields.setdefault("is_superuser", True)
        return self.create_user(email, password, **extra_fields)


class User(AbstractUser):
    username = None  # 移除 username 字段
    email = models.EmailField(unique=True)
    phone = models.CharField(max_length=20, blank=True)
    avatar = models.ImageField(upload_to="avatars/", blank=True)

    USERNAME_FIELD = "email"
    REQUIRED_FIELDS = []

    objects = UserManager()

    def __str__(self):
        return self.email
```

### Groups 和 Permissions

```python
from django.contrib.auth.models import Group, Permission
from django.contrib.contenttypes.models import ContentType

# 创建自定义权限
class Article(models.Model):
    class Meta:
        permissions = [
            ("publish_article", "Can publish article"),
            ("feature_article", "Can feature article on homepage"),
        ]

# 程序化管理权限
def setup_groups():
    editors, _ = Group.objects.get_or_create(name="Editors")
    content_type = ContentType.objects.get_for_model(Article)

    publish_perm = Permission.objects.get(
        codename="publish_article", content_type=content_type
    )
    editors.permissions.add(publish_perm)

# 检查权限
user.has_perm("myapp.publish_article")
user.has_perms(["myapp.publish_article", "myapp.change_article"])
```

### JWT 认证（djangorestframework-simplejwt）

```python
# settings.py
from datetime import timedelta

INSTALLED_APPS += ["rest_framework_simplejwt"]

REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework_simplejwt.authentication.JWTAuthentication",
    ],
}

SIMPLE_JWT = {
    "ACCESS_TOKEN_LIFETIME": timedelta(minutes=15),
    "REFRESH_TOKEN_LIFETIME": timedelta(days=7),
    "ROTATE_REFRESH_TOKENS": True,
    "BLACKLIST_AFTER_ROTATION": True,
    "AUTH_HEADER_TYPES": ("Bearer",),
    "TOKEN_OBTAIN_SERIALIZER": "apps.users.serializers.CustomTokenObtainPairSerializer",
}

# urls.py
from rest_framework_simplejwt.views import TokenObtainPairView, TokenRefreshView

urlpatterns = [
    path("api/token/", TokenObtainPairView.as_view(), name="token_obtain_pair"),
    path("api/token/refresh/", TokenRefreshView.as_view(), name="token_refresh"),
]

# 自定义 Token 载荷
from rest_framework_simplejwt.serializers import TokenObtainPairSerializer

class CustomTokenObtainPairSerializer(TokenObtainPairSerializer):
    @classmethod
    def get_token(cls, user):
        token = super().get_token(user)
        token["email"] = user.email
        token["is_staff"] = user.is_staff
        return token
```

---

## Django REST Framework

### Serializer

```python
from rest_framework import serializers

class ArticleSerializer(serializers.ModelSerializer):
    author_name = serializers.CharField(source="author.get_full_name", read_only=True)
    category_name = serializers.CharField(source="category.name", read_only=True)
    comment_count = serializers.IntegerField(read_only=True)

    class Meta:
        model = Article
        fields = [
            "id", "title", "slug", "body", "status",
            "author", "author_name",
            "category", "category_name",
            "publish_date", "view_count", "comment_count",
            "created_at", "updated_at",
        ]
        read_only_fields = ["author", "view_count"]
        extra_kwargs = {
            "body": {"min_length": 50},
        }

    def validate_title(self, value):
        if Article.objects.filter(title=value).exclude(pk=self.instance and self.instance.pk).exists():
            raise serializers.ValidationError("Title already exists.")
        return value

    def validate(self, data):
        if data.get("status") == "published" and not data.get("category"):
            raise serializers.ValidationError(
                {"category": "Published articles must have a category."}
            )
        return data

    def create(self, validated_data):
        validated_data["author"] = self.context["request"].user
        return super().create(validated_data)


class ArticleListSerializer(serializers.ModelSerializer):
    """列表专用轻量 Serializer，减少数据传输"""
    class Meta:
        model = Article
        fields = ["id", "title", "slug", "status", "publish_date", "view_count"]
```

### ViewSet

```python
from rest_framework import viewsets, status
from rest_framework.decorators import action
from rest_framework.response import Response
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework.filters import SearchFilter, OrderingFilter


class ArticleViewSet(viewsets.ModelViewSet):
    queryset = Article.objects.select_related("author", "category")
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ["status", "category", "author"]
    search_fields = ["title", "body"]
    ordering_fields = ["publish_date", "view_count"]
    ordering = ["-publish_date"]

    def get_serializer_class(self):
        if self.action == "list":
            return ArticleListSerializer
        return ArticleSerializer

    def get_queryset(self):
        qs = super().get_queryset()
        if self.action == "list":
            qs = qs.annotate(comment_count=Count("comments"))
        return qs

    @action(detail=True, methods=["post"])
    def publish(self, request, pk=None):
        article = self.get_object()
        article.status = "published"
        article.publish_date = timezone.now()
        article.save(update_fields=["status", "publish_date"])
        return Response({"status": "published"})

    @action(detail=False, methods=["get"])
    def my_articles(self, request):
        qs = self.get_queryset().filter(author=request.user)
        page = self.paginate_queryset(qs)
        if page is not None:
            serializer = self.get_serializer(page, many=True)
            return self.get_paginated_response(serializer.data)
        serializer = self.get_serializer(qs, many=True)
        return Response(serializer.data)
```

### Permission

```python
from rest_framework.permissions import BasePermission, SAFE_METHODS


class IsAuthorOrReadOnly(BasePermission):
    def has_object_permission(self, request, view, obj):
        if request.method in SAFE_METHODS:
            return True
        return obj.author == request.user


class IsAdminOrEditor(BasePermission):
    def has_permission(self, request, view):
        return request.user.is_staff or request.user.groups.filter(name="Editors").exists()


# 在 ViewSet 中组合使用
class ArticleViewSet(viewsets.ModelViewSet):
    permission_classes = [IsAuthenticated, IsAuthorOrReadOnly]

    def get_permissions(self):
        if self.action in ["publish", "destroy"]:
            return [IsAuthenticated(), IsAdminOrEditor()]
        return super().get_permissions()
```

### Pagination

```python
# settings.py
REST_FRAMEWORK = {
    "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
    "PAGE_SIZE": 20,
}

# 自定义分页
from rest_framework.pagination import CursorPagination

class ArticleCursorPagination(CursorPagination):
    page_size = 20
    ordering = "-publish_date"
    cursor_query_param = "cursor"

class ArticleViewSet(viewsets.ModelViewSet):
    pagination_class = ArticleCursorPagination
```

### Throttling

```python
# settings.py
REST_FRAMEWORK = {
    "DEFAULT_THROTTLE_CLASSES": [
        "rest_framework.throttling.AnonRateThrottle",
        "rest_framework.throttling.UserRateThrottle",
    ],
    "DEFAULT_THROTTLE_RATES": {
        "anon": "100/hour",
        "user": "1000/hour",
        "burst": "60/minute",
    },
}

from rest_framework.throttling import UserRateThrottle

class BurstRateThrottle(UserRateThrottle):
    scope = "burst"

class ArticleViewSet(viewsets.ModelViewSet):
    throttle_classes = [BurstRateThrottle]
```

---

## 中间件

### 自定义中间件

```python
import time
import logging

logger = logging.getLogger(__name__)


class RequestTimingMiddleware:
    """记录每次请求的耗时"""
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        start = time.monotonic()
        response = self.get_response(request)
        duration = time.monotonic() - start
        response["X-Request-Duration"] = f"{duration:.4f}s"
        if duration > 1.0:
            logger.warning(
                "Slow request: %s %s took %.2fs",
                request.method, request.path, duration,
            )
        return response


class RequestIDMiddleware:
    """为每次请求添加唯一 ID，方便日志追踪"""
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        import uuid
        request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
        request.request_id = request_id
        response = self.get_response(request)
        response["X-Request-ID"] = request_id
        return response


# settings.py
MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "corsheaders.middleware.CorsMiddleware",
    "apps.core.middleware.RequestIDMiddleware",
    "apps.core.middleware.RequestTimingMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
]
```

---

## 信号系统

### 信号定义与使用

```python
# apps/users/signals.py
from django.db.models.signals import post_save, pre_save
from django.dispatch import receiver
from django.conf import settings


@receiver(post_save, sender=settings.AUTH_USER_MODEL)
def create_user_profile(sender, instance, created, **kwargs):
    if created:
        Profile.objects.create(user=instance)


@receiver(pre_save, sender="articles.Article")
def auto_slug(sender, instance, **kwargs):
    if not instance.slug:
        from django.utils.text import slugify
        instance.slug = slugify(instance.title)
```

### 信号注册

```python
# apps/users/apps.py
from django.apps import AppConfig

class UsersConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "apps.users"

    def ready(self):
        import apps.users.signals  # noqa: F401
```

### 信号使用原则

- 信号适合跨 app 通知（如用户注册后发邮件）
- 不要用信号做本 app 内的业务逻辑，应放在 Model 方法或 Service 层
- 信号中避免耗时操作，长任务交给 Celery
- 测试时可用 `signal.disconnect()` 临时解除

---

## 缓存策略

### Redis 缓存配置

```python
# settings.py
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": os.environ.get("REDIS_URL", "redis://127.0.0.1:6379/1"),
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
            "SERIALIZER": "django_redis.serializers.json.JSONSerializer",
            "CONNECTION_POOL_KWARGS": {"max_connections": 50},
            "SOCKET_CONNECT_TIMEOUT": 5,
            "SOCKET_TIMEOUT": 5,
        },
        "KEY_PREFIX": "myproject",
        "TIMEOUT": 300,  # 默认 5 分钟
    }
}

# 使用 Redis 做 Session 后端
SESSION_ENGINE = "django.contrib.sessions.backends.cache"
SESSION_CACHE_ALIAS = "default"
```

### Per-View 缓存

```python
from django.views.decorators.cache import cache_page
from django.utils.decorators import method_decorator

# 函数视图
@cache_page(60 * 15)  # 缓存 15 分钟
def article_list(request):
    ...

# 类视图
@method_decorator(cache_page(60 * 15), name="dispatch")
class ArticleListView(ListView):
    ...

# DRF ViewSet —— 使用 vary_on_headers 区分用户
from django.views.decorators.vary import vary_on_headers

@method_decorator(cache_page(60 * 5), name="list")
@method_decorator(vary_on_headers("Authorization"), name="list")
class ArticleViewSet(viewsets.ModelViewSet):
    ...
```

### Per-Site 缓存

```python
MIDDLEWARE = [
    "django.middleware.cache.UpdateCacheMiddleware",      # 放在最前
    # ... 其他中间件 ...
    "django.middleware.cache.FetchFromCacheMiddleware",   # 放在最后
]

CACHE_MIDDLEWARE_ALIAS = "default"
CACHE_MIDDLEWARE_SECONDS = 600
CACHE_MIDDLEWARE_KEY_PREFIX = "site"
```

### 低级缓存 API

```python
from django.core.cache import cache

# 基本操作
cache.set("article:123", article_data, timeout=300)
data = cache.get("article:123")
cache.delete("article:123")

# get_or_set 模式
def get_article_stats():
    return Article.objects.aggregate(
        total=Count("id"),
        published=Count("id", filter=Q(status="published")),
    )

stats = cache.get_or_set("article_stats", get_article_stats, timeout=600)

# 批量操作
cache.set_many({"key1": "val1", "key2": "val2"}, timeout=300)
data = cache.get_many(["key1", "key2"])

# 原子递增
cache.set("page_views:123", 0)
cache.incr("page_views:123")

# 缓存版本控制
cache.set("data", value, version=2)
cache.incr_version("data")

# 缓存失效策略
def invalidate_article_cache(article_id):
    keys = [
        f"article:{article_id}",
        "article_list",
        "article_stats",
    ]
    cache.delete_many(keys)
```

---

## Celery 异步任务

### 基础配置

```python
# config/celery.py
import os
from celery import Celery

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "config.settings.prod")

app = Celery("myproject")
app.config_from_object("django.conf:settings", namespace="CELERY")
app.autodiscover_tasks()

# settings.py
CELERY_BROKER_URL = os.environ.get("CELERY_BROKER_URL", "redis://127.0.0.1:6379/0")
CELERY_RESULT_BACKEND = "django-db"  # django-celery-results
CELERY_ACCEPT_CONTENT = ["json"]
CELERY_TASK_SERIALIZER = "json"
CELERY_RESULT_SERIALIZER = "json"
CELERY_TIMEZONE = "Asia/Shanghai"
CELERY_TASK_TRACK_STARTED = True
CELERY_TASK_TIME_LIMIT = 600
CELERY_TASK_SOFT_TIME_LIMIT = 300
CELERY_WORKER_MAX_TASKS_PER_CHILD = 1000
CELERY_WORKER_PREFETCH_MULTIPLIER = 1
```

### 任务定义与重试

```python
# apps/notifications/tasks.py
from celery import shared_task
from celery.utils.log import get_task_logger

logger = get_task_logger(__name__)


@shared_task(
    bind=True,
    max_retries=3,
    default_retry_delay=60,
    autoretry_for=(ConnectionError, TimeoutError),
    retry_backoff=True,
    retry_backoff_max=600,
    retry_jitter=True,
)
def send_notification_email(self, user_id, template_name, context):
    try:
        user = User.objects.get(id=user_id)
        send_mail(
            subject=context["subject"],
            message="",
            html_message=render_to_string(template_name, context),
            from_email=settings.DEFAULT_FROM_EMAIL,
            recipient_list=[user.email],
        )
        logger.info("Email sent to user %s", user_id)
    except User.DoesNotExist:
        logger.error("User %s not found, not retrying", user_id)
    except Exception as exc:
        logger.warning("Email failed for user %s: %s", user_id, exc)
        raise self.retry(exc=exc)
```

### 任务编排

```python
from celery import chain, group, chord

# chain —— 串行执行
result = chain(
    fetch_data.s(url),
    process_data.s(),
    store_results.s(),
)()

# group —— 并行执行
result = group(
    process_image.s(image_id) for image_id in image_ids
)()

# chord —— 并行执行 + 汇总回调
result = chord(
    [analyze_article.s(aid) for aid in article_ids],
    aggregate_results.s(),
)()
```

### 任务优先级

```python
# 定义队列
CELERY_TASK_ROUTES = {
    "apps.notifications.tasks.*": {"queue": "notifications"},
    "apps.analytics.tasks.*": {"queue": "analytics", "priority": 3},
    "apps.orders.tasks.*": {"queue": "critical", "priority": 9},
}

# 手动指定队列
send_notification_email.apply_async(
    args=[user_id, "welcome.html", ctx],
    queue="notifications",
    priority=5,
    countdown=10,  # 延迟 10 秒执行
)
```

### Celery Beat 定时任务

```python
from celery.schedules import crontab

CELERY_BEAT_SCHEDULE = {
    "cleanup-expired-sessions": {
        "task": "apps.core.tasks.cleanup_expired_sessions",
        "schedule": crontab(hour=3, minute=0),  # 每天凌晨 3 点
    },
    "generate-daily-report": {
        "task": "apps.analytics.tasks.generate_daily_report",
        "schedule": crontab(hour=6, minute=30),
    },
    "sync-external-data": {
        "task": "apps.integrations.tasks.sync_external_data",
        "schedule": 300.0,  # 每 5 分钟
    },
}
```

---

## 安全

### CSRF 防护

```python
# settings.py
CSRF_COOKIE_SECURE = True            # 仅 HTTPS 传输 CSRF cookie
CSRF_COOKIE_HTTPONLY = True           # JS 不可读取
CSRF_TRUSTED_ORIGINS = [
    "https://mysite.com",
    "https://*.mysite.com",
]

# DRF API 中排除 CSRF（JWT 场景）
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework_simplejwt.authentication.JWTAuthentication",
    ],
    # SessionAuthentication 会强制 CSRF，纯 JWT 时不要包含
}
```

### XSS 防护

```python
# Django 模板默认自动转义，确保不要滥用 |safe 过滤器
# 在 DRF 中 JSON 响应天然不存在模板 XSS

# 清理用户输入的 HTML
import bleach

ALLOWED_TAGS = ["p", "b", "i", "u", "a", "ul", "ol", "li", "br", "strong", "em"]
ALLOWED_ATTRS = {"a": ["href", "title"]}

def sanitize_html(raw_html):
    return bleach.clean(raw_html, tags=ALLOWED_TAGS, attributes=ALLOWED_ATTRS, strip=True)
```

### SQL 注入防护

```python
# Django ORM 自动参数化，以下是安全的
Article.objects.filter(title=user_input)
Article.objects.extra(where=["title=%s"], params=[user_input])  # 参数化

# 危险 —— 永远不要这样做
Article.objects.raw(f"SELECT * FROM article WHERE title='{user_input}'")

# 安全的 raw SQL
Article.objects.raw("SELECT * FROM article WHERE title=%s", [user_input])

# 安全使用 connection.cursor
from django.db import connection
with connection.cursor() as cursor:
    cursor.execute("SELECT * FROM article WHERE status=%s", ["published"])
```

### Content Security Policy

```python
# 使用 django-csp
# pip install django-csp
MIDDLEWARE += ["csp.middleware.CSPMiddleware"]

CSP_DEFAULT_SRC = ("'self'",)
CSP_SCRIPT_SRC = ("'self'", "cdn.jsdelivr.net")
CSP_STYLE_SRC = ("'self'", "'unsafe-inline'", "fonts.googleapis.com")
CSP_FONT_SRC = ("'self'", "fonts.gstatic.com")
CSP_IMG_SRC = ("'self'", "data:", "cdn.mysite.com")
CSP_CONNECT_SRC = ("'self'", "api.mysite.com")
```

### 综合安全设置

```python
# settings/prod.py
SECURE_HSTS_SECONDS = 31536000
SECURE_HSTS_INCLUDE_SUBDOMAINS = True
SECURE_HSTS_PRELOAD = True
SECURE_SSL_REDIRECT = True
SECURE_BROWSER_XSS_FILTER = True
SECURE_CONTENT_TYPE_NOSNIFF = True
SESSION_COOKIE_SECURE = True
SESSION_COOKIE_HTTPONLY = True
SESSION_COOKIE_AGE = 3600  # 1 小时
X_FRAME_OPTIONS = "DENY"
```

---

## 测试

### TestCase 与 APITestCase

```python
from django.test import TestCase, TransactionTestCase
from rest_framework.test import APITestCase, APIClient
from django.urls import reverse


class ArticleModelTest(TestCase):
    @classmethod
    def setUpTestData(cls):
        """类级别数据准备，整个 TestCase 共享，速度更快"""
        cls.user = User.objects.create_user(
            email="test@example.com", password="testpass123"
        )
        cls.category = Category.objects.create(name="Tech", slug="tech")

    def test_article_creation(self):
        article = Article.objects.create(
            title="Test Article",
            body="x" * 100,
            author=self.user,
            category=self.category,
        )
        self.assertEqual(str(article), "Test Article")
        self.assertEqual(article.status, Article.Status.DRAFT)

    def test_published_manager(self):
        Article.objects.create(
            title="Draft", body="x" * 100,
            author=self.user, status="draft",
        )
        Article.objects.create(
            title="Published", body="x" * 100,
            author=self.user, status="published",
        )
        self.assertEqual(Article.published.count(), 1)


class ArticleAPITest(APITestCase):
    def setUp(self):
        self.user = User.objects.create_user(
            email="api@example.com", password="testpass123"
        )
        self.client = APIClient()
        self.client.force_authenticate(user=self.user)

    def test_create_article(self):
        url = reverse("article-list")
        data = {
            "title": "API Article",
            "body": "x" * 100,
            "status": "draft",
        }
        response = self.client.post(url, data, format="json")
        self.assertEqual(response.status_code, 201)
        self.assertEqual(Article.objects.count(), 1)
        self.assertEqual(Article.objects.first().author, self.user)

    def test_list_articles_pagination(self):
        for i in range(25):
            Article.objects.create(
                title=f"Article {i}", body="x" * 100, author=self.user
            )
        response = self.client.get(reverse("article-list"))
        self.assertEqual(response.status_code, 200)
        self.assertEqual(len(response.data["results"]), 20)

    def test_unauthorized_delete(self):
        other_user = User.objects.create_user(email="other@example.com", password="pass")
        article = Article.objects.create(
            title="Other", body="x" * 100, author=other_user
        )
        url = reverse("article-detail", kwargs={"pk": article.pk})
        response = self.client.delete(url)
        self.assertEqual(response.status_code, 403)
```

### Factory Boy

```python
# apps/articles/tests/factories.py
import factory
from factory.django import DjangoModelFactory
from apps.users.models import User
from apps.articles.models import Article, Category


class UserFactory(DjangoModelFactory):
    class Meta:
        model = User

    email = factory.Sequence(lambda n: f"user{n}@example.com")
    password = factory.PostGenerationMethodCall("set_password", "testpass123")
    is_active = True


class CategoryFactory(DjangoModelFactory):
    class Meta:
        model = Category

    name = factory.Sequence(lambda n: f"Category {n}")
    slug = factory.LazyAttribute(lambda o: o.name.lower().replace(" ", "-"))


class ArticleFactory(DjangoModelFactory):
    class Meta:
        model = Article

    title = factory.Sequence(lambda n: f"Article {n}")
    slug = factory.LazyAttribute(lambda o: o.title.lower().replace(" ", "-"))
    body = factory.Faker("paragraph", nb_sentences=10)
    author = factory.SubFactory(UserFactory)
    category = factory.SubFactory(CategoryFactory)
    status = "draft"

    class Params:
        published = factory.Trait(
            status="published",
            publish_date=factory.LazyFunction(timezone.now),
        )

# 在测试中使用
class ArticleTest(TestCase):
    def test_with_factory(self):
        article = ArticleFactory(published=True)
        self.assertEqual(article.status, "published")

    def test_batch_create(self):
        ArticleFactory.create_batch(10, published=True)
        self.assertEqual(Article.published.count(), 10)
```

### 覆盖率配置

```ini
# .coveragerc 或 pyproject.toml
[tool.coverage.run]
source = ["apps"]
omit = ["*/migrations/*", "*/tests/*", "*/admin.py"]
branch = true

[tool.coverage.report]
fail_under = 85
show_missing = true
exclude_lines = [
    "pragma: no cover",
    "def __repr__",
    "raise NotImplementedError",
    "if TYPE_CHECKING:",
]
```

```bash
pytest --cov=apps --cov-report=term-missing --cov-report=html
```

---

## 部署

### Gunicorn 配置

```python
# gunicorn.conf.py
import multiprocessing

bind = "0.0.0.0:8000"
workers = multiprocessing.cpu_count() * 2 + 1
worker_class = "gthread"
threads = 2
worker_tmp_dir = "/dev/shm"
timeout = 30
graceful_timeout = 30
keepalive = 5
max_requests = 1000
max_requests_jitter = 50
accesslog = "-"
errorlog = "-"
loglevel = "info"
preload_app = True
```

### Nginx 配置

```nginx
upstream django {
    server 127.0.0.1:8000;
}

server {
    listen 80;
    server_name mysite.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name mysite.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    client_max_body_size 10M;

    location /static/ {
        alias /app/staticfiles/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    location /media/ {
        alias /app/media/;
        expires 7d;
    }

    location / {
        proxy_pass http://django;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 30s;
        proxy_connect_timeout 10s;
    }
}
```

### Docker 部署

```dockerfile
# Dockerfile
FROM python:3.12-slim AS base

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev gcc && \
    rm -rf /var/lib/apt/lists/*

COPY requirements/prod.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

COPY . .
RUN python manage.py collectstatic --noinput

FROM base AS production
RUN addgroup --system django && adduser --system --group django
USER django

CMD ["gunicorn", "config.wsgi:application", "-c", "gunicorn.conf.py"]
```

```yaml
# docker-compose.yml
services:
  web:
    build: .
    command: gunicorn config.wsgi:application -c gunicorn.conf.py
    volumes:
      - static_volume:/app/staticfiles
      - media_volume:/app/media
    env_file: .env
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_started

  db:
    image: postgres:16-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: ${DB_NAME}
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER}"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru

  celery:
    build: .
    command: celery -A config worker -l info --concurrency=4
    env_file: .env
    depends_on: [web, redis]

  celery-beat:
    build: .
    command: celery -A config beat -l info --scheduler django_celery_beat.schedulers:DatabaseScheduler
    env_file: .env
    depends_on: [web, redis]

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
      - static_volume:/app/staticfiles
      - media_volume:/app/media

volumes:
  postgres_data:
  static_volume:
  media_volume:
```

### Kubernetes 部署

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: django-web
spec:
  replicas: 3
  selector:
    matchLabels:
      app: django-web
  template:
    metadata:
      labels:
        app: django-web
    spec:
      containers:
        - name: web
          image: myregistry/django-app:latest
          ports:
            - containerPort: 8000
          envFrom:
            - secretRef:
                name: django-secrets
            - configMapRef:
                name: django-config
          resources:
            requests:
              cpu: "250m"
              memory: "256Mi"
            limits:
              cpu: "1000m"
              memory: "512Mi"
          readinessProbe:
            httpGet:
              path: /health/
              port: 8000
            initialDelaySeconds: 10
            periodSeconds: 5
          livenessProbe:
            httpGet:
              path: /health/
              port: 8000
            initialDelaySeconds: 30
            periodSeconds: 15
```

---

## 性能优化

### 数据库查询优化

```python
# 1. 使用 select_related 减少 ForeignKey 查询
# 差: 每篇文章额外查询 author 和 category
articles = Article.objects.all()
for a in articles:
    print(a.author.email)  # N+1 查询!

# 好: 单次 JOIN 查询
articles = Article.objects.select_related("author", "category").all()

# 2. 使用 prefetch_related 处理 ManyToMany
articles = Article.objects.prefetch_related("tags").all()

# 3. 只查需要的字段
Article.objects.values_list("id", "title", flat=False)
Article.objects.only("id", "title", "status")
Article.objects.defer("body")  # 延迟加载大字段

# 4. 使用 iterator() 处理大结果集
for article in Article.objects.all().iterator(chunk_size=2000):
    process(article)

# 5. 批量操作
Article.objects.bulk_create([
    Article(title=f"Article {i}", body="...", author=user)
    for i in range(1000)
], batch_size=500)

Article.objects.filter(status="draft").update(status="archived")

Article.objects.bulk_update(articles, ["status", "updated_at"], batch_size=500)
```

### 索引策略

```python
class Article(models.Model):
    class Meta:
        indexes = [
            # 单字段索引
            models.Index(fields=["status"]),
            # 复合索引 —— 查询条件的列顺序要与索引一致
            models.Index(fields=["status", "-publish_date"]),
            # 部分索引（PostgreSQL）
            models.Index(
                fields=["publish_date"],
                condition=Q(status="published"),
                name="idx_published_date",
            ),
            # 覆盖索引（PostgreSQL，Django 5.0+）
            models.Index(
                fields=["status"],
                include=["title", "publish_date"],
                name="idx_status_covering",
            ),
        ]
```

### 查询调试

```python
# 在开发环境中追踪查询
from django.db import connection, reset_queries

reset_queries()
# ... 执行查询 ...
print(f"Total queries: {len(connection.queries)}")
for q in connection.queries:
    print(f"[{q['time']}s] {q['sql'][:200]}")

# 使用 django-debug-toolbar
INSTALLED_APPS += ["debug_toolbar"]
MIDDLEWARE += ["debug_toolbar.middleware.DebugToolbarMiddleware"]
INTERNAL_IPS = ["127.0.0.1"]

# QuerySet.explain() 查看执行计划
qs = Article.objects.filter(status="published").order_by("-publish_date")
print(qs.explain(analyze=True))
```

---

## 常见陷阱

### N+1 查询

```python
# 陷阱: 模板或序列化器中触发隐式查询
# 检测方法: django-debug-toolbar / django-silk / nplusone

# 解决方案: 在 QuerySet 层提前 JOIN 或预取
# View 层负责定义 QuerySet，Serializer 不应触发额外查询

class ArticleViewSet(viewsets.ModelViewSet):
    def get_queryset(self):
        return (
            Article.objects
            .select_related("author", "category")
            .prefetch_related("tags", "comments__author")
            .annotate(comment_count=Count("comments"))
        )
```

### 信号滥用

```python
# 陷阱: 把业务逻辑放在信号中
# - 隐式执行，难以调试和追踪
# - 多个信号之间可能存在顺序依赖
# - 信号异常可能被吞掉

# 解决方案: 使用 Service 层替代
class ArticleService:
    @staticmethod
    def publish(article):
        article.status = "published"
        article.publish_date = timezone.now()
        article.save(update_fields=["status", "publish_date"])
        # 明确的后续操作
        send_notification_email.delay(article.author_id, "article_published.html", {...})
        cache.delete(f"article:{article.id}")
        return article
```

### Fat Models 反模式

```python
# 陷阱: Model 中包含大量业务逻辑、外部调用、邮件发送
class Article(models.Model):
    def publish(self):
        self.status = "published"
        self.save()
        send_mail(...)         # 模型不应直接发邮件
        requests.post(...)     # 模型不应调用外部服务
        cache.delete(...)      # 模型不应管理缓存

# 解决方案: 将业务逻辑抽到 Service 层
# Model 只负责: 字段定义、Meta、__str__、简单计算属性、自定义 Manager
# Service 层负责: 业务流程、外部调用、缓存、通知
```

### 其他常见陷阱

```python
# 1. 在循环中 save() —— 应使用 bulk_update
for article in articles:
    article.view_count += 1
    article.save()  # 每次都是一条 UPDATE!

# 修正
Article.objects.filter(id__in=ids).update(view_count=F("view_count") + 1)

# 2. 未正确使用事务
from django.db import transaction

# 差: 部分成功部分失败
def transfer_funds(from_account, to_account, amount):
    from_account.balance -= amount
    from_account.save()
    # 如果这里崩溃，钱就消失了!
    to_account.balance += amount
    to_account.save()

# 好: 原子事务
@transaction.atomic
def transfer_funds(from_account, to_account, amount):
    from_account.balance = F("balance") - amount
    from_account.save(update_fields=["balance"])
    to_account.balance = F("balance") + amount
    to_account.save(update_fields=["balance"])

# 3. settings.py 中硬编码秘密信息
SECRET_KEY = "hardcoded-secret"  # 永远不要这样做!
SECRET_KEY = os.environ["DJANGO_SECRET_KEY"]  # 从环境变量读取

# 4. 迁移中使用当前模型代码
# 差: 迁移运行时模型可能已变
from apps.users.models import User

# 好: 使用历史模型
def forward(apps, schema_editor):
    User = apps.get_model("users", "User")
```

---

## Agent Checklist

以下是 Super Dev Agent 在 Django 项目中执行审查和交付时必须验证的检查清单:

### 项目结构
- [ ] 使用 settings 拆分（base/dev/staging/prod）
- [ ] `AUTH_USER_MODEL` 在第一次迁移前已定义
- [ ] 每个 app 有独立的 `tests/` 目录和 `factories.py`
- [ ] requirements 按环境拆分（base/dev/prod）

### ORM 与数据库
- [ ] 所有 ForeignKey 查询使用 `select_related`
- [ ] 所有 ManyToMany / 反向查询使用 `prefetch_related`
- [ ] 列表接口使用 `only()` / `defer()` 排除大字段
- [ ] 批量操作使用 `bulk_create` / `bulk_update` 而非循环 save
- [ ] 高频查询字段有 `db_index=True` 或 Meta.indexes
- [ ] 关键写操作包裹在 `transaction.atomic` 中
- [ ] 迁移中使用 `apps.get_model()` 而非直接 import

### 认证与权限
- [ ] 自定义 User 模型继承 `AbstractUser` 或 `AbstractBaseUser`
- [ ] JWT 配置了 token 过期时间和刷新轮换
- [ ] API ViewSet 显式声明 `permission_classes`
- [ ] 对象级权限已实现（如 IsAuthorOrReadOnly）

### REST API
- [ ] 列表和详情使用不同 Serializer（轻量 vs 完整）
- [ ] 分页已配置（推荐 CursorPagination 用于大数据集）
- [ ] 限流已配置（区分匿名和认证用户）
- [ ] 输入验证在 Serializer 层完成（validate_field / validate）

### 缓存
- [ ] Redis 连接池大小和超时已配置
- [ ] 缓存 key 有命名规范和前缀
- [ ] 数据变更时有缓存失效策略
- [ ] 不缓存包含用户敏感信息的响应

### 异步任务
- [ ] Celery 任务配置了 `max_retries` 和 `time_limit`
- [ ] 使用 `retry_backoff=True` 避免重试风暴
- [ ] 任务参数可 JSON 序列化（不传 Model 实例，传 ID）
- [ ] 定时任务通过 Beat 管理，不使用 cron 直调

### 安全
- [ ] 生产环境 `DEBUG=False`
- [ ] `SECRET_KEY` 从环境变量读取
- [ ] HSTS/SSL/Secure Cookie 已启用
- [ ] CSRF 保护正确配置（Session 认证开启，纯 JWT 可排除）
- [ ] 用户输入 HTML 经过 `bleach` 清理
- [ ] Raw SQL 使用参数化查询
- [ ] CSP 头已配置

### 测试
- [ ] 单元测试覆盖率 >= 85%
- [ ] 使用 Factory Boy 而非手动创建测试数据
- [ ] API 测试覆盖认证、权限、分页、错误场景
- [ ] 使用 `setUpTestData` 替代 `setUp` 提升速度

### 部署
- [ ] Gunicorn workers 数量 = CPU * 2 + 1
- [ ] Nginx 配置静态文件直接服务和缓存头
- [ ] Docker 镜像使用非 root 用户运行
- [ ] 健康检查端点 `/health/` 已实现
- [ ] 数据库连接配置 `CONN_MAX_AGE` 和超时
- [ ] Celery worker 配置了 `max_tasks_per_child` 防止内存泄漏
