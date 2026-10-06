package com.openforum.plugin;

import com.openforum.entity.SubForum;

public interface RecommendationPlugin extends ForumPlugin {

    double score(SubForum forum);
}
