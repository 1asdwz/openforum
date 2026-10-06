package com.openforum.plugin;

import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ReviewMethod;

import java.util.Map;

public interface ReviewPlugin extends ForumPlugin {

    /**
     * @return {@code true} when the plugin is usable under the current configuration.
     */
    boolean isAvailable();

    ReviewMethod getReviewMethod();

    ReviewResult review(AuthType type, Map<String, Object> data);
}
