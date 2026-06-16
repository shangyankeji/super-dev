---
id: file-upload-s3-playbook
title: 文件上传实战手册（S3/MinIO）
domain: backend
category: 02-playbooks
difficulty: advanced
tags: [file-upload, s3, minio, presigned-url, multipart, virus-scan, thumbnail, cdn, backend, enterprise]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# 文件上传实战手册（S3 / MinIO）

## 上传模式对比

| 模式 | 流程 | 适合 | 限制 |
|------|------|------|------|
| 直传后端 | 前端 → 后端 → S3 | 小文件（< 5MB） | 后端带宽瓶颈 |
| Presigned URL | 前端 → S3 直传 | 大文件（推荐） | 后端不经手 |
| Multipart | 分片直传 S3 | 超大文件（> 100MB） | 复杂 |

## Presigned URL（推荐）

```python
# 后端生成 presigned URL
@app.post("/api/upload/sign")
def create_presigned_upload(filename: str, content_type: str, user=Depends(get_current_user)):
    # 1. 生成唯一 key（防覆盖）
    key = f"uploads/{user.id}/{uuid.uuid4()}/{filename}"

    # 2. 校验类型（白名单）
    allowed = {"image/jpeg", "image/png", "image/webp", "application/pdf"}
    if content_type not in allowed:
        raise HTTPException(400, "File type not allowed")

    # 3. 生成 presigned PUT URL（5 分钟有效）
    url = s3.generate_presigned_url(
        "put_object",
        Params={
            "Bucket": "my-bucket",
            "Key": key,
            "ContentType": content_type,
        },
        ExpiresIn=300,
    )

    # 4. 返回 URL + key（前端直传 S3）
    return {"uploadUrl": url, "key": key, "expiresIn": 300}
```

```typescript
// 前端直传 S3（不经过后端）
async function uploadFile(file: File) {
  // 1. 向后端获取 presigned URL
  const { uploadUrl, key } = await api.post('/api/upload/sign', {
    filename: file.name,
    contentType: file.type,
  });

  // 2. 直接 PUT 到 S3
  await fetch(uploadUrl, {
    method: 'PUT',
    headers: { 'Content-Type': file.type },
    body: file,
  });

  // 3. 通知后端上传完成（记录 key）
  await api.post('/api/upload/confirm', { key });
}
```

## 安全

### 文件类型验证（多层）
```python
# ❌ 只检查扩展名（不安全）
if filename.endswith('.jpg'):  # 改名就能绕过

# ✅ 多层验证
# 1. Content-Type 白名单
allowed_types = {"image/jpeg", "image/png", "image/webp"}

# 2. Magic bytes（文件头）验证
import magic
real_type = magic.from_buffer(file_content, mime=True)
if real_type not in allowed_types:
    raise HTTPException(400, "File type mismatch")

# 3. 文件大小限制（presigned URL 配置）
# 4. 文件名消毒（防路径穿越）
safe_name = os.path.basename(filename)  # 去掉 ../
key = f"uploads/{uuid.uuid4()}/{safe_name}"
```

### 病毒扫描
```python
# ClamAV 集成（后台异步扫描）
@celery.task
def scan_file(key: str):
    obj = s3.get_object(Bucket="my-bucket", Key=key)
    result = clamav.scan_stream(obj["Body"])
    if result.infected:
        s3.delete_object(Bucket="my-bucket", Key=key)  # 删除感染文件
        db.update(File, key, status="rejected")
    else:
        db.update(File, key, status="clean")
```

## 图片处理

```python
# 上传后生成缩略图（异步）
@celery.task
def generate_thumbnails(key: str):
    original = s3.get_object(Bucket="my-bucket", Key=key)["Body"].read()
    for size in [(150, 150), (400, 400), (800, 800)]:
        thumb = PIL.Image.open(io.BytesIO(original))
        thumb.thumbnail(size)
        buf = io.BytesIO()
        thumb.save(buf, format="WEBP", quality=85)  # 转 WebP 省空间
        thumb_key = key.replace("uploads/", f"thumbs/{size[0]}x{size[0]}/")
        s3.put_object(Bucket="my-bucket", Key=thumb_key, Body=buf.getvalue(), ContentType="image/webp")
```

## CDN 分发

```yaml
# CloudFront / Cloudflare 分发
# 用户上传后 → S3 → CloudFront CDN → 全球加速分发
Distribution:
  Origins:
    - S3Origin:
        DomainName: my-bucket.s3.amazonaws.com
  DefaultCacheBehavior:
    ViewerProtocolPolicy: redirect-to-https
    CachePolicyId: managed-CachingOptimized
    # 图片缓存 1 年（内容不变）
    MinTTL: 0
    DefaultTTL: 31536000
```

## 生产检查清单
- [ ] 用 Presigned URL 直传 S3（不经后端）
- [ ] Content-Type 白名单 + Magic Bytes 验证
- [ ] 文件大小限制（presigned URL + 前端）
- [ ] UUID 文件 key（防覆盖/路径穿越）
- [ ] 异步病毒扫描（ClamAV）
- [ ] 图片自动生成缩略图（WebP 格式）
- [ ] CDN 分发静态文件
- [ ] CORS 配置（S3 bucket 只允许前端域名）
- [ ] 生命周期策略（自动删除临时文件）
- [ ] 上传进度条（前端 UX）
