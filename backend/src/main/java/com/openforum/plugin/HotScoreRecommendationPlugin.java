package com.openforum.plugin;

import com.openforum.entity.Post;
import com.openforum.entity.SubForum;
import com.openforum.repository.PostRepository;
import org.springframework.stereotype.Component;

import java.time.LocalDateTime;
import java.util.List;

@Component
public class HotScoreRecommendationPlugin implements RecommendationPlugin {

    private final PostRepository postRepository;

    public HotScoreRecommendationPlugin(PostRepository postRepository) {
        this.postRepository = postRepository;
    }

    @Override
    public String getKey() {
        return "hot-score-recommendation";
    }

    @Override
    public String getName() {
        return "热度推荐";
    }

    @Override
    public PluginType getType() {
        return PluginType.RECOMMENDATION;
    }

    @Override
    public boolean isEnabledByDefault() {
        return true;
    }

    @Override
    public String getDescription() {
        return "按成员数 / 帖子数 / 近 7 天活跃度计算子论坛推荐分";
    }

    @Override
    public double score(SubForum forum) {
        double base = forum.getMemberCount() * 2.0 + forum.getPostCount() * 3.0;
        List<Post> posts = postRepository.findByForumIdAndRemovedFalse(forum.getId());
        LocalDateTime weekAgo = LocalDateTime.now().minusDays(7);
        long recent = posts.stream()
                .filter(post -> post.getCreatedAt() != null && post.getCreatedAt().isAfter(weekAgo))
                .count();
        long voteScore = posts.stream().mapToLong(Post::getScore).sum();
        return base + recent * 5.0 + voteScore * 0.1;
    }
}
