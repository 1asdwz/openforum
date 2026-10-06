package com.openforum.service;

import com.openforum.config.OpenForumAuthProperties;
import com.openforum.dto.authentication.AuthMediaUploadResponse;
import com.openforum.entity.AuthMedia;
import com.openforum.entity.enums.MediaKind;
import com.openforum.exception.ApiException;
import com.openforum.plugin.DocumentReviewPlugin;
import com.openforum.repository.AuthMediaRepository;
import com.openforum.service.storage.StorageService;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.web.multipart.MultipartFile;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.security.MessageDigest;
import java.time.LocalDate;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * 认证媒体上传/读取/绑定。字节存于私有对象存储，仅经鉴权接口读取。
 */
@Service
public class AuthMediaService {

    private static final Set<String> IMAGE_TYPES = Set.of("image/jpeg", "image/png", "image/webp");
    private static final Set<String> VIDEO_TYPES = Set.of("video/webm", "video/mp4");
    private static final Map<String, String> EXTENSIONS = Map.of(
            "image/jpeg", "jpg",
            "image/png", "png",
            "image/webp", "webp",
            "video/webm", "webm",
            "video/mp4", "mp4");

    private final AuthMediaRepository mediaRepository;
    private final StorageService storageService;
    private final OpenForumAuthProperties properties;

    public AuthMediaService(AuthMediaRepository mediaRepository, StorageService storageService,
                            OpenForumAuthProperties properties) {
        this.mediaRepository = mediaRepository;
        this.storageService = storageService;
        this.properties = properties;
    }

    @Transactional
    public AuthMediaUploadResponse upload(MultipartFile file, String kindValue, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        if (file == null || file.isEmpty()) {
            throw ApiException.badRequest("上传文件不能为空");
        }
        if (!storageService.isEnabled()) {
            throw ApiException.badRequest("文件存储未启用，无法上传认证材料");
        }
        MediaKind kind = parseKind(kindValue);
        String contentType = file.getContentType() == null ? "" : file.getContentType().toLowerCase();
        long size = file.getSize();
        if (kind == MediaKind.VIDEO) {
            if (!VIDEO_TYPES.contains(contentType)) {
                throw ApiException.badRequest("视频格式不支持，仅允许 webm/mp4");
            }
            if (size > properties.maxVideoBytes()) {
                throw ApiException.badRequest("视频过大，上限 " + (properties.maxVideoBytes() / 1024 / 1024) + "MB");
            }
        } else {
            if (!IMAGE_TYPES.contains(contentType)) {
                throw ApiException.badRequest("图片格式不支持，仅允许 jpeg/png/webp");
            }
            if (size > properties.maxImageBytes()) {
                throw ApiException.badRequest("图片过大，上限 " + (properties.maxImageBytes() / 1024 / 1024) + "MB");
            }
        }

        byte[] bytes;
        try {
            bytes = file.getBytes();
        } catch (Exception ex) {
            throw ApiException.badRequest("读取上传文件失败");
        }
        String extension = EXTENSIONS.getOrDefault(contentType, "bin");
        String objectKey = "auth/" + userId + "/"
                + LocalDate.now().format(DateTimeFormatter.ofPattern("yyyyMMdd")) + "/"
                + UUID.randomUUID() + "." + extension;
        storageService.put(objectKey, new ByteArrayInputStream(bytes), bytes.length, contentType);

        AuthMedia media = new AuthMedia();
        media.setUserId(userId);
        media.setKind(kind);
        media.setObjectKey(objectKey);
        media.setContentType(contentType);
        media.setSizeBytes(size);
        media.setSha256(sha256(bytes));
        mediaRepository.save(media);

        return new AuthMediaUploadResponse(media.getId(), kind.name(), contentType, size);
    }

    /** 校验归属并绑定到认证记录。 */
    @Transactional
    public List<AuthMedia> attach(List<Long> mediaIds, Long authenticationId, Long userId) {
        if (mediaIds == null || mediaIds.isEmpty()) {
            return List.of();
        }
        List<AuthMedia> media = mediaRepository.findByUserIdAndIdIn(userId, mediaIds);
        if (media.size() != mediaIds.size()) {
            throw ApiException.badRequest("存在无效或不属于当前用户的媒体文件");
        }
        for (AuthMedia item : media) {
            item.setAuthenticationId(authenticationId);
        }
        return mediaRepository.saveAll(media);
    }

    /** 下载字节供插件使用。 */
    public List<DocumentReviewPlugin.MediaRef> materialize(List<Long> mediaIds, Long userId) {
        if (mediaIds == null || mediaIds.isEmpty()) {
            return List.of();
        }
        List<AuthMedia> media = mediaRepository.findByUserIdAndIdIn(userId, mediaIds);
        if (media.size() != mediaIds.size()) {
            throw ApiException.badRequest("存在无效或不属于当前用户的媒体文件");
        }
        List<DocumentReviewPlugin.MediaRef> refs = new ArrayList<>();
        for (AuthMedia item : media) {
            byte[] bytes = readBytes(item.getObjectKey());
            refs.add(new DocumentReviewPlugin.MediaRef(
                    item.getId(), item.getKind(), item.getContentType(), bytes));
        }
        return refs;
    }

    public MediaResource load(Long mediaId, Long userId, boolean admin) {
        AuthMedia media = mediaRepository.findById(mediaId)
                .orElseThrow(() -> ApiException.notFound("媒体文件不存在"));
        if (!admin && !media.getUserId().equals(userId)) {
            throw ApiException.forbidden("无权访问该媒体文件");
        }
        if (!storageService.isEnabled()) {
            throw ApiException.badRequest("文件存储未启用");
        }
        InputStream stream = storageService.get(media.getObjectKey());
        return new MediaResource(stream, media.getContentType(), media.getSizeBytes());
    }

    public Optional<AuthMedia> find(Long mediaId) {
        return mediaRepository.findById(mediaId);
    }

    private MediaKind parseKind(String value) {
        if (value == null || value.isBlank()) {
            throw ApiException.badRequest("文件类型 kind 不能为空");
        }
        try {
            return MediaKind.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            throw ApiException.badRequest("未知的文件类型: " + value);
        }
    }

    private byte[] readBytes(String objectKey) {
        try (InputStream in = storageService.get(objectKey)) {
            return in.readAllBytes();
        } catch (ApiException ex) {
            throw ex;
        } catch (Exception ex) {
            throw ApiException.badRequest("读取媒体文件失败");
        }
    }

    private String sha256(byte[] bytes) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(bytes);
            StringBuilder sb = new StringBuilder(hash.length * 2);
            for (byte b : hash) {
                sb.append(String.format("%02x", b));
            }
            return sb.toString();
        } catch (Exception ex) {
            return null;
        }
    }

    public record MediaResource(InputStream stream, String contentType, long size) {
    }
}
