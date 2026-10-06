package com.openforum.service;

import com.openforum.common.PageResponse;
import com.openforum.dto.post.PostCreateRequest;
import com.openforum.dto.post.PostDetailDTO;
import com.openforum.dto.post.PostSummaryDTO;
import com.openforum.dto.post.PostUpdateRequest;
import com.openforum.dto.user.AuthorDTO;
import com.openforum.entity.Post;
import com.openforum.entity.SubForum;
import com.openforum.entity.User;
import com.openforum.entity.enums.VoteTargetType;
import com.openforum.exception.ApiException;
import com.openforum.repository.PostRepository;
import com.openforum.repository.SubForumRepository;
import com.openforum.repository.UserRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.Duration;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

@Service
public class PostService {

    private final PostRepository postRepository;
    private final SubForumRepository subForumRepository;
    private final UserRepository userRepository;
    private final VoteService voteService;
    private final AuthGateService gateService;
    private final ForumAccessService accessService;

    public PostService(PostRepository postRepository, SubForumRepository subForumRepository,
                       UserRepository userRepository, VoteService voteService,
                       AuthGateService gateService, ForumAccessService accessService) {
        this.postRepository = postRepository;
        this.subForumRepository = subForumRepository;
        this.userRepository = userRepository;
        this.voteService = voteService;
        this.gateService = gateService;
        this.accessService = accessService;
    }

    public PageResponse<PostSummaryDTO> list(Long forumId, String sort, int page, int size, Long userId) {
        List<Post> posts = forumId == null
                ? new ArrayList<>(postRepository.findByRemovedFalse())
                : new ArrayList<>(postRepository.findByForumIdAndRemovedFalse(forumId));
        Comparator<Post> comparator = switch (sort == null ? "hot" : sort.toLowerCase()) {
            case "new" -> Comparator.comparing(Post::getCreatedAt).reversed();
            case "top" -> Comparator.comparingInt(Post::getScore).reversed();
            default -> Comparator.comparingDouble(this::hotScore).reversed();
        };
        posts.sort(Comparator.comparing(Post::isPinned).reversed().thenComparing(comparator));

        int total = posts.size();
        int from = Math.max(0, Math.min(page * size, total));
        int to = Math.min(from + size, total);
        List<PostSummaryDTO> content = posts.subList(from, to).stream().map(this::toSummary).toList();
        return PageResponse.of(content, page, size, total);
    }

    public PostDetailDTO detail(Long postId, Long userId) {
        return toDetail(require(postId), userId);
    }

    @Transactional
    public PostDetailDTO create(PostCreateRequest request, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        SubForum forum = subForumRepository.findById(request.forumId())
                .orElseThrow(() -> ApiException.notFound("子论坛不存在"));
        AuthGateService.GateResult gate = gateService.evaluate(userId, forum.getRequiredAuthTypes());
        if (!gate.passed()) {
            throw ApiException.forbidden(gate.message());
        }
        Post post = new Post();
        post.setForumId(forum.getId());
        post.setAuthorId(userId);
        post.setTitle(request.title());
        post.setContent(request.content());
        postRepository.save(post);

        forum.setPostCount((int) postRepository.countByForumIdAndRemovedFalse(forum.getId()));
        subForumRepository.save(forum);
        return toDetail(post, userId);
    }

    @Transactional
    public PostDetailDTO update(Long postId, PostUpdateRequest request, Long userId) {
        Post post = require(postId);
        assertCanEdit(post, userId);
        if (request.title() != null && !request.title().isBlank()) {
            post.setTitle(request.title());
        }
        if (request.content() != null) {
            post.setContent(request.content());
        }
        postRepository.save(post);
        return toDetail(post, userId);
    }

    @Transactional
    public void delete(Long postId, Long userId) {
        Post post = require(postId);
        assertCanEdit(post, userId);
        post.setRemoved(true);
        postRepository.save(post);
        voteService.clearVotes(VoteTargetType.POST, postId);

        subForumRepository.findById(post.getForumId()).ifPresent(forum -> {
            forum.setPostCount((int) postRepository.countByForumIdAndRemovedFalse(forum.getId()));
            subForumRepository.save(forum);
        });
    }

    @Transactional
    public PostDetailDTO vote(Long postId, Integer value, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        Post post = require(postId);
        SubForum forum = subForumRepository.findById(post.getForumId())
                .orElseThrow(() -> ApiException.notFound("子论坛不存在"));
        AuthGateService.GateResult gate = gateService.evaluate(userId, forum.getRequiredAuthTypes());
        if (!gate.passed()) {
            throw ApiException.forbidden(gate.message());
        }
        int voteValue = value == null ? 0 : value;
        if (voteValue != -1 && voteValue != 0 && voteValue != 1) {
            throw ApiException.badRequest("投票值只能是 1、-1 或 0");
        }
        VoteService.VoteOutcome outcome = voteService.apply(userId, VoteTargetType.POST, postId, voteValue,
                post.getUpvotes(), post.getDownvotes());
        post.setUpvotes(outcome.upvotes());
        post.setDownvotes(outcome.downvotes());
        post.setScore(outcome.score());
        postRepository.save(post);
        return toDetail(post, userId);
    }

    @Transactional
    public PostDetailDTO pin(Long postId, Long userId) {
        Post post = require(postId);
        SubForum forum = subForumRepository.findById(post.getForumId())
                .orElseThrow(() -> ApiException.notFound("子论坛不存在"));
        if (!accessService.canManage(forum, userId)) {
            throw ApiException.forbidden("需要坛主或管理员权限");
        }
        post.setPinned(!post.isPinned());
        postRepository.save(post);
        return toDetail(post, userId);
    }

    private Post require(Long postId) {
        return postRepository.findByIdAndRemovedFalse(postId)
                .orElseThrow(() -> ApiException.notFound("帖子不存在"));
    }

    private void assertCanEdit(Post post, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        boolean isAuthor = userId.equals(post.getAuthorId());
        boolean isManager = subForumRepository.findById(post.getForumId())
                .map(forum -> accessService.canManage(forum, userId))
                .orElse(false);
        if (!isAuthor && !isManager) {
            throw ApiException.forbidden("只有作者或坛主可以操作");
        }
    }

    private double hotScore(Post post) {
        double base = post.getScore() + post.getCommentCount() * 2.0;
        if (post.getCreatedAt() == null) {
            return base;
        }
        long days = Duration.between(post.getCreatedAt(), LocalDateTime.now()).toDays();
        double recency = days < 7 ? Math.max(0, 7 - days) * 5.0 : 0;
        return base + recency;
    }

    private PostSummaryDTO toSummary(Post post) {
        SubForum forum = subForumRepository.findById(post.getForumId()).orElse(null);
        return new PostSummaryDTO(post.getId(), post.getForumId(),
                forum == null ? null : forum.getSlug(),
                forum == null ? null : forum.getName(),
                post.getTitle(), preview(post.getContent()),
                post.getUpvotes(), post.getDownvotes(), post.getScore(), post.getCommentCount(),
                post.isPinned(), author(post.getAuthorId()), post.getCreatedAt());
    }

    private PostDetailDTO toDetail(Post post, Long userId) {
        SubForum forum = subForumRepository.findById(post.getForumId()).orElse(null);
        boolean canVote = userId != null && forum != null
                && gateService.evaluate(userId, forum.getRequiredAuthTypes()).passed();
        Integer myVote = userId == null ? 0 : voteService.myVote(userId, VoteTargetType.POST, post.getId());
        return new PostDetailDTO(post.getId(), post.getForumId(),
                forum == null ? null : forum.getSlug(),
                forum == null ? null : forum.getName(),
                post.getTitle(), post.getContent(),
                post.getUpvotes(), post.getDownvotes(), post.getScore(), post.getCommentCount(),
                post.isPinned(), author(post.getAuthorId()), canVote, myVote,
                post.getCreatedAt(), post.getUpdatedAt());
    }

    private AuthorDTO author(Long userId) {
        User user = userRepository.findById(userId).orElse(null);
        return user == null ? null : new AuthorDTO(user.getId(), user.getUsername(), user.getAvatar());
    }

    private String preview(String content) {
        if (content == null) {
            return null;
        }
        String trimmed = content.strip();
        return trimmed.length() <= 200 ? trimmed : trimmed.substring(0, 200) + "…";
    }
}
