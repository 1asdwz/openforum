package com.openforum.service;

import com.openforum.dto.comment.CommentCreateRequest;
import com.openforum.dto.comment.CommentDTO;
import com.openforum.entity.Comment;
import com.openforum.entity.Post;
import com.openforum.entity.SubForum;
import com.openforum.entity.User;
import com.openforum.entity.enums.VoteTargetType;
import com.openforum.exception.ApiException;
import com.openforum.repository.CommentRepository;
import com.openforum.repository.PostRepository;
import com.openforum.repository.SubForumRepository;
import com.openforum.repository.UserRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Service
public class CommentService {

    private final CommentRepository commentRepository;
    private final PostRepository postRepository;
    private final SubForumRepository subForumRepository;
    private final UserRepository userRepository;
    private final ForumAccessService accessService;
    private final VoteService voteService;

    public CommentService(CommentRepository commentRepository, PostRepository postRepository,
                          SubForumRepository subForumRepository, UserRepository userRepository,
                          ForumAccessService accessService, VoteService voteService) {
        this.commentRepository = commentRepository;
        this.postRepository = postRepository;
        this.subForumRepository = subForumRepository;
        this.userRepository = userRepository;
        this.accessService = accessService;
        this.voteService = voteService;
    }

    public List<CommentDTO> tree(Long postId) {
        List<Comment> comments = commentRepository.findByPostIdAndRemovedFalseOrderByCreatedAtAsc(postId);
        Map<Long, List<Comment>> byParent = new HashMap<>();
        List<Comment> roots = new ArrayList<>();
        for (Comment comment : comments) {
            if (comment.getParentId() == null) {
                roots.add(comment);
            } else {
                byParent.computeIfAbsent(comment.getParentId(), key -> new ArrayList<>()).add(comment);
            }
        }
        return roots.stream().map(comment -> build(comment, byParent)).toList();
    }

    @Transactional
    public CommentDTO create(CommentCreateRequest request, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        Post post = postRepository.findByIdAndRemovedFalse(request.postId())
                .orElseThrow(() -> ApiException.notFound("帖子不存在"));
        if (request.parentId() != null) {
            Comment parent = commentRepository.findByIdAndRemovedFalse(request.parentId())
                    .orElseThrow(() -> ApiException.badRequest("父评论不存在"));
            if (!parent.getPostId().equals(request.postId())) {
                throw ApiException.badRequest("父评论不属于该帖子");
            }
        }
        Comment comment = new Comment();
        comment.setPostId(request.postId());
        comment.setAuthorId(userId);
        comment.setParentId(request.parentId());
        comment.setContent(request.content());
        commentRepository.save(comment);

        post.setCommentCount((int) commentRepository.countByPostIdAndRemovedFalse(post.getId()));
        postRepository.save(post);
        return build(comment, Map.of());
    }

    @Transactional
    public void delete(Long commentId, Long userId) {
        Comment comment = commentRepository.findByIdAndRemovedFalse(commentId)
                .orElseThrow(() -> ApiException.notFound("评论不存在"));
        Post post = postRepository.findById(comment.getPostId()).orElse(null);
        SubForum forum = post == null ? null
                : subForumRepository.findById(post.getForumId()).orElse(null);
        boolean isAuthor = userId != null && userId.equals(comment.getAuthorId());
        boolean isManager = forum != null && accessService.canManage(forum, userId);
        if (!isAuthor && !isManager) {
            throw ApiException.forbidden("只有作者或坛主可以删除评论");
        }
        comment.setRemoved(true);
        commentRepository.save(comment);
        voteService.clearVotes(VoteTargetType.COMMENT, commentId);

        if (post != null) {
            post.setCommentCount((int) commentRepository.countByPostIdAndRemovedFalse(post.getId()));
            postRepository.save(post);
        }
    }

    @Transactional
    public CommentDTO vote(Long commentId, Integer value, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        Comment comment = commentRepository.findByIdAndRemovedFalse(commentId)
                .orElseThrow(() -> ApiException.notFound("评论不存在"));
        int voteValue = value == null ? 0 : value;
        if (voteValue != -1 && voteValue != 0 && voteValue != 1) {
            throw ApiException.badRequest("投票值只能是 1、-1 或 0");
        }
        VoteService.VoteOutcome outcome = voteService.apply(userId, VoteTargetType.COMMENT, commentId, voteValue,
                comment.getUpvotes(), comment.getDownvotes());
        comment.setUpvotes(outcome.upvotes());
        comment.setDownvotes(outcome.downvotes());
        comment.setScore(outcome.score());
        commentRepository.save(comment);
        return build(comment, Map.of());
    }

    private CommentDTO build(Comment comment, Map<Long, List<Comment>> byParent) {
        List<CommentDTO> children = byParent.getOrDefault(comment.getId(), List.of()).stream()
                .map(child -> build(child, byParent))
                .toList();
        User author = userRepository.findById(comment.getAuthorId()).orElse(null);
        return new CommentDTO(comment.getId(), comment.getPostId(), comment.getAuthorId(),
                author == null ? null : author.getUsername(),
                author == null ? null : author.getAvatar(),
                comment.getParentId(), comment.getContent(),
                comment.getUpvotes(), comment.getDownvotes(), comment.getScore(),
                comment.getCreatedAt(), children);
    }
}
