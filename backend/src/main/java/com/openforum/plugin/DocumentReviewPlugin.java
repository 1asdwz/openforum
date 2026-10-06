package com.openforum.plugin;

import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.MediaKind;
import com.openforum.entity.enums.ReviewMethod;

import java.util.List;
import java.util.Map;

/**
 * 证件/视频专业审核插件（MEDIUM+）。
 */
public interface DocumentReviewPlugin extends ForumPlugin {

    boolean isAvailable();

    ReviewMethod getReviewMethod();

    ReviewResult reviewDocuments(DocumentReviewContext ctx);

    /**
     * 交给插件的媒体引用：由服务层下载字节后传入，插件不直接访问存储。
     */
    record MediaRef(Long id, MediaKind kind, String contentType, byte[] bytes) {
    }

    record DocumentReviewContext(
            AuthType type,
            AuthLevel level,
            Map<String, Object> data,
            List<MediaRef> documents,
            MediaRef video,
            byte[] videoPoster) {

        public DocumentReviewContext {
            if (data == null) {
                data = Map.of();
            }
            if (documents == null) {
                documents = List.of();
            }
        }
    }
}
