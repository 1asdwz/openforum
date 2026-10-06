package com.openforum.service;

import com.openforum.config.OpenForumAuthProperties;
import com.openforum.dto.media.MediaUploadResponse;
import com.openforum.entity.PostMedia;
import com.openforum.exception.ApiException;
import com.openforum.repository.PostMediaRepository;
import com.openforum.service.storage.StorageService;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.web.multipart.MultipartFile;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.time.LocalDate;
import java.time.format.DateTimeFormatter;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

@Service
public class PostMediaService {

    private static final Set<String> IMAGE_TYPES = Set.of(
            "image/jpeg", "image/png", "image/webp", "image/gif");
    private static final Set<String> VIDEO_TYPES = Set.of(
            "video/mp4", "video/webm");
    private static final Map<String, String> EXTENSIONS = Map.of(
            "image/jpeg", "jpg",
            "image/png", "png",
            "image/webp", "webp",
            "image/gif", "gif",
            "video/mp4", "mp4",
            "video/webm", "webm");

    private final PostMediaRepository mediaRepository;
    private final StorageService storageService;
    private final OpenForumAuthProperties properties;

    public PostMediaService(PostMediaRepository mediaRepository,
                            StorageService storageService,
                            OpenForumAuthProperties properties) {
        this.mediaRepository = mediaRepository;
        this.storageService = storageService;
        this.properties = properties;
    }

    @Transactional
    public MediaUploadResponse upload(MultipartFile file, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        if (file == null || file.isEmpty()) {
            throw ApiException.badRequest("上传文件不能为空");
        }
        if (!storageService.isEnabled()) {
            throw ApiException.badRequest("文件存储未启用，无法上传媒体");
        }
        String contentType = file.getContentType() == null ? "" : file.getContentType().toLowerCase();
        long size = file.getSize();
        if (VIDEO_TYPES.contains(contentType)) {
            if (size > properties.maxVideoBytes()) {
                throw ApiException.badRequest("视频过大，上限 " + properties.maxVideoBytes() / 1024 / 1024 + "MB");
            }
        } else if (IMAGE_TYPES.contains(contentType)) {
            if (size > properties.maxImageBytes()) {
                throw ApiException.badRequest("图片过大，上限 " + properties.maxImageBytes() / 1024 / 1024 + "MB");
            }
        } else {
            throw ApiException.badRequest("文件格式不支持，图片仅允许 jpeg/png/webp/gif，视频仅允许 mp4/webm");
        }

        byte[] bytes;
        try {
            bytes = file.getBytes();
        } catch (Exception ex) {
            throw ApiException.badRequest("读取上传文件失败");
        }
        String extension = EXTENSIONS.getOrDefault(contentType, "bin");
        String objectKey = "post/" + userId + "/"
                + LocalDate.now().format(DateTimeFormatter.ofPattern("yyyyMMdd"))
                + "/" + UUID.randomUUID() + "." + extension;
        storageService.put(objectKey, new ByteArrayInputStream(bytes), bytes.length, contentType);

        PostMedia media = new PostMedia();
        media.setUserId(userId);
        media.setObjectKey(objectKey);
        media.setContentType(contentType);
        media.setSizeBytes(size);
        mediaRepository.save(media);
        return new MediaUploadResponse(media.getId(), contentType, size);
    }

    public MediaResource load(Long id) {
        PostMedia media = mediaRepository.findById(id)
                .orElseThrow(() -> ApiException.notFound("媒体文件不存在"));
        if (!storageService.isEnabled()) {
            throw ApiException.badRequest("文件存储未启用");
        }
        InputStream stream = storageService.get(media.getObjectKey());
        long size = media.getSizeBytes() == null ? 0L : media.getSizeBytes();
        return new MediaResource(stream, media.getContentType(), size);
    }

    public record MediaResource(InputStream stream, String contentType, long size) {
    }
}
