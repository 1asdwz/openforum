package com.openforum.dto.post;

import com.openforum.dto.user.AuthorDTO;

import java.time.LocalDateTime;

public class PostDetailDTO {

    private final Long id;
    private final Long forumId;
    private final String forumSlug;
    private final String forumName;
    private final String title;
    private final String content;
    private final int upvotes;
    private final int downvotes;
    private final int score;
    private final int commentCount;
    private final boolean isPinned;
    private final AuthorDTO author;
    private final boolean canVote;
    private final Integer myVote;
    private final LocalDateTime createdAt;
    private final LocalDateTime updatedAt;

    public PostDetailDTO(Long id, Long forumId, String forumSlug, String forumName, String title, String content,
                         int upvotes, int downvotes, int score, int commentCount, boolean isPinned, AuthorDTO author,
                         boolean canVote, Integer myVote, LocalDateTime createdAt, LocalDateTime updatedAt) {
        this.id = id;
        this.forumId = forumId;
        this.forumSlug = forumSlug;
        this.forumName = forumName;
        this.title = title;
        this.content = content;
        this.upvotes = upvotes;
        this.downvotes = downvotes;
        this.score = score;
        this.commentCount = commentCount;
        this.isPinned = isPinned;
        this.author = author;
        this.canVote = canVote;
        this.myVote = myVote;
        this.createdAt = createdAt;
        this.updatedAt = updatedAt;
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

    public String getContent() {
        return content;
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

    public boolean getCanVote() {
        return canVote;
    }

    public Integer getMyVote() {
        return myVote;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public LocalDateTime getUpdatedAt() {
        return updatedAt;
    }
}
