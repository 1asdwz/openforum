package com.openforum.repository;

import com.openforum.entity.Comment;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface CommentRepository extends JpaRepository<Comment, Long> {

    Optional<Comment> findByIdAndRemovedFalse(Long id);

    List<Comment> findByPostIdAndRemovedFalseOrderByCreatedAtAsc(Long postId);

    List<Comment> findByParentIdAndRemovedFalseOrderByCreatedAtAsc(Long parentId);

    long countByPostIdAndRemovedFalse(Long postId);

    List<Comment> findByAuthorIdAndRemovedFalse(Long authorId);
}
