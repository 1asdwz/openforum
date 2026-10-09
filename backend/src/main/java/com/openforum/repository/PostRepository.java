package com.openforum.repository;

import com.openforum.entity.Post;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface PostRepository extends JpaRepository<Post, Long> {

    Optional<Post> findByIdAndRemovedFalse(Long id);

    Page<Post> findByForumIdAndRemovedFalse(Long forumId, Pageable pageable);

    Page<Post> findByRemovedFalse(Pageable pageable);

    List<Post> findByForumIdAndRemovedFalse(Long forumId);

    List<Post> findByRemovedFalse();

    List<Post> findByAuthorIdAndRemovedFalse(Long authorId);

    List<Post> findByTitleContainingIgnoreCaseAndRemovedFalse(String title);

    long countByForumIdAndRemovedFalse(Long forumId);
}
