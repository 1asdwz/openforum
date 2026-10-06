package com.openforum.repository;

import com.openforum.entity.ForumMember;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface ForumMemberRepository extends JpaRepository<ForumMember, Long> {

    Optional<ForumMember> findByForumIdAndUserId(Long forumId, Long userId);

    boolean existsByForumIdAndUserId(Long forumId, Long userId);

    List<ForumMember> findByForumIdOrderByJoinedAtAsc(Long forumId);

    List<ForumMember> findByUserIdOrderByJoinedAtDesc(Long userId);

    long countByForumId(Long forumId);

    void deleteByForumIdAndUserId(Long forumId, Long userId);
}
