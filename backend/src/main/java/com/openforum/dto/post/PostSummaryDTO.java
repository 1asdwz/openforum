package com.openforum.dto.post;

import com.openforum.dto.user.AuthorDTO;

import java.time.LocalDateTime;

public class PostSummaryDTO {

    private final Long id;
    private final Long forumId;
    private final String forumSlug;
    private final String forumName;
    private final String title;
    private final String contentPreview;
    private final int upvotes;
    private final int downvotes;
    private final int score;
    private final int commentCount;
    private final boolean isPinned;
    private final AuthorDTO author;
    private final LocalDateTime createdAt;

    public PostSummaryDTO(Long id, Long forumId, String forumSlug, String forumName, String title, String contentPreview,
                          int upvotes, int downvotes, int score, int commentCount, boolean isPinned, AuthorDTO author,
                          LocalDateTime createdAt) {
        this.id = id;
        this.forumId = forumId;
        this.forumSlug = forumSlug;
        this.forumName = forumName;
        this.title = title;
        this.contentPreview = contentPreview;
        this.upvotes = upvotes;
        this.downvotes = downvotes;
        this.score = score;
        this.commentCount = commentCount;
        this.isPinned = isPinned;
        this.author = author;
        this.createdAt = createdAt;
    }

    public Long getId() {
        return id;
    }

    public Long getForumId() {
        return forumId;
    }

    public String getForumSlug() {
        return forumSlug;
    }

    public String getForumName() {
        return forumName;
    }

    public String getTitle() {
        return title;
    }

    public String getContentPreview() {
        return contentPreview;
    }

    public int getUpvotes() {
        return upvotes;
    }

    public int getDownvotes() {
        return downvotes;
    }

    public int getScore() {
        return score;
    }

    public int getCommentCount() {
        return commentCount;
    }

    public boolean getIsPinned() {
        return isPinned;
    }

    public AuthorDTO getAuthor() {
        return author;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }
}
